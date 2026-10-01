/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Scenario runner implementation
//!
//! Provides both standalone (stdout) and Buck test (JSON) execution modes.

use crate::logging::log_message;
use crate::resources::ResourceState;
use crate::resources::STATE_FILE_NAME;
use crate::scenario::Scenario;
use crate::scenario::Test;

// ANSI color codes for terminal output
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Mode for test execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerMode {
    /// Standalone mode: all output to stdout/stderr, human-readable format
    Standalone,
    /// Buck test mode: JSON output format for Buck test infrastructure
    BuckTest,
}

/// Configuration for the scenario runner
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Execution mode
    pub mode: RunnerMode,
    /// Optional filter for scenario names
    pub scenario_filter: Option<String>,
    /// Optional filter for test names
    pub test_filter: Option<String>,
    /// Whether to stop on first failure
    pub fail_fast: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            mode: RunnerMode::Standalone,
            scenario_filter: None,
            test_filter: None,
            fail_fast: false,
        }
    }
}

impl RunnerConfig {
    pub fn standalone() -> Self {
        Self {
            mode: RunnerMode::Standalone,
            ..Default::default()
        }
    }

    pub fn buck_test() -> Self {
        Self {
            mode: RunnerMode::BuckTest,
            ..Default::default()
        }
    }

    pub fn with_scenario_filter(mut self, filter: impl Into<String>) -> Self {
        self.scenario_filter = Some(filter.into());
        self
    }

    pub fn with_test_filter(mut self, filter: impl Into<String>) -> Self {
        self.test_filter = Some(filter.into());
        self
    }

    pub fn with_fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }
}

/// Status of a test after execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed(String),
    Skipped(String),
}

impl TestStatus {
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

/// Result of a single test execution
#[derive(Debug)]
pub struct TestRunResult {
    pub scenario: String,
    pub test: String,
    pub status: TestStatus,
}

/// Summary of a test run
#[derive(Debug, Default)]
pub struct RunSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub results: Vec<TestRunResult>,
}

impl RunSummary {
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Scenario runner that executes tests
///
/// Distinguishes between resources that were never provisioned (skip)
/// and resources whose setup was attempted but failed (fail).
enum ResourceCheckResult {
    Skipped(String),
    Failed(String),
}

/// Scenario runner that executes tests
pub struct ScenarioRunner {
    config: RunnerConfig,
    scenarios: Vec<Scenario>,
}

impl ScenarioRunner {
    pub fn new(config: RunnerConfig) -> Self {
        Self {
            config,
            scenarios: Vec::new(),
        }
    }

    pub fn standalone() -> Self {
        Self::new(RunnerConfig::standalone())
    }

    pub fn buck_test() -> Self {
        Self::new(RunnerConfig::buck_test())
    }

    /// Add a scenario to the runner
    pub fn add_scenario(&mut self, scenario: Scenario) {
        self.scenarios.push(scenario);
    }

    /// Add multiple scenarios to the runner
    pub fn add_scenarios(&mut self, scenarios: impl IntoIterator<Item = Scenario>) {
        self.scenarios.extend(scenarios);
    }

    fn matches_scenario_filter(&self, name: &str) -> bool {
        match &self.config.scenario_filter {
            Some(filter) => {
                // Support comma-separated filters (e.g., "shell,sync,pip")
                filter.split(',').any(|f| name.contains(f.trim()))
            }
            None => true,
        }
    }

    fn matches_test_filter(&self, name: &str) -> bool {
        match &self.config.test_filter {
            Some(filter) => name.contains(filter),
            None => true,
        }
    }

    /// Run all scenarios and return a summary
    pub fn run(&self) -> RunSummary {
        let mut summary = RunSummary::default();

        match self.config.mode {
            RunnerMode::Standalone => self.run_standalone(&mut summary),
            RunnerMode::BuckTest => self.run_buck_test(&mut summary),
        }

        summary
    }

    fn run_standalone(&self, summary: &mut RunSummary) {
        eprintln!("{}=== Scenario Test Runner ==={}\n", BOLD, RESET);

        let scenarios: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| self.matches_scenario_filter(s.name()))
            .collect();

        eprintln!("Running {} scenario(s)...\n", scenarios.len());

        for scenario in scenarios {
            eprintln!("{}━━━ Scenario: {} ━━━{}", BOLD, scenario.name(), RESET);

            // Check resources at scenario level
            if let Err(reason) = scenario.check_resources() {
                eprintln!(
                    "{}⏭️  SKIPPED{} (resources unavailable: {})\n",
                    YELLOW, RESET, reason
                );
                for test in scenario.tests() {
                    if self.matches_test_filter(test.name) {
                        summary.skipped += 1;
                        summary.results.push(TestRunResult {
                            scenario: scenario.name().to_string(),
                            test: test.name.to_string(),
                            status: TestStatus::Skipped(reason.clone()),
                        });
                    }
                }
                continue;
            }

            for test in scenario.tests() {
                if !self.matches_test_filter(test.name) {
                    continue;
                }

                let _full_name = format!("{}::{}", scenario.name(), test.name);
                eprintln!("  Running: {} ...", test.name);

                let status = self.run_test(test);
                match &status {
                    TestStatus::Passed => {
                        eprintln!("  {}✓ PASSED{}\n", GREEN, RESET);
                        summary.passed += 1;
                    }
                    TestStatus::Failed(e) => {
                        eprintln!("  {}✗ FAILED{}: {}\n", RED, RESET, e);
                        summary.failed += 1;
                    }
                    TestStatus::Skipped(reason) => {
                        eprintln!("  {}⏭️  SKIPPED{}: {}\n", YELLOW, RESET, reason);
                        summary.skipped += 1;
                    }
                }

                summary.results.push(TestRunResult {
                    scenario: scenario.name().to_string(),
                    test: test.name.to_string(),
                    status: status.clone(),
                });

                if self.config.fail_fast && status.is_failed() {
                    eprintln!("Stopping due to fail-fast mode");
                    return;
                }
            }
        }

        // Print detailed test results report
        self.print_test_results_report(summary);
    }

    fn print_test_results_report(&self, summary: &RunSummary) {
        eprintln!(
            "\n{}═══════════════════════════════════════════════════════════════════════════════{}",
            BOLD, RESET
        );
        eprintln!(
            "{}                              TEST RESULTS REPORT                              {}",
            BOLD, RESET
        );
        eprintln!(
            "{}═══════════════════════════════════════════════════════════════════════════════{}\n",
            BOLD, RESET
        );

        // Print passed tests
        if summary.passed > 0 {
            eprintln!("{}{}✓ PASSED ({}):{}", BOLD, GREEN, summary.passed, RESET);
            for result in &summary.results {
                if result.status.is_passed() {
                    eprintln!("  {}✓{} {}::{}", GREEN, RESET, result.scenario, result.test);
                }
            }
            eprintln!();
        }

        // Print failed tests
        if summary.failed > 0 {
            eprintln!("{}{}✗ FAILED ({}):{}", BOLD, RED, summary.failed, RESET);
            for result in &summary.results {
                if let TestStatus::Failed(ref e) = result.status {
                    eprintln!(
                        "  {}✗{} {}::{}: {}",
                        RED, RESET, result.scenario, result.test, e
                    );
                }
            }
            eprintln!();
        }

        // Print skipped tests
        if summary.skipped > 0 {
            eprintln!(
                "{}{}⏭️  SKIPPED ({}):{}",
                BOLD, YELLOW, summary.skipped, RESET
            );
            for result in &summary.results {
                if let TestStatus::Skipped(ref reason) = result.status {
                    eprintln!(
                        "  {}⏭️{} {}::{}: {}",
                        YELLOW, RESET, result.scenario, result.test, reason
                    );
                }
            }
            eprintln!();
        }

        // Print summary line
        eprintln!(
            "{}───────────────────────────────────────────────────────────────────────────────{}",
            BOLD, RESET
        );
        eprintln!(
            "{}Total: {} | {}Passed: {}{} | {}Failed: {}{} | {}Skipped: {}{}",
            BOLD,
            summary.total(),
            GREEN,
            summary.passed,
            RESET,
            RED,
            summary.failed,
            RESET,
            YELLOW,
            summary.skipped,
            RESET
        );

        if summary.all_passed() {
            eprintln!("\n{}{}🎉 All tests passed!{}", BOLD, GREEN, RESET);
        } else {
            eprintln!("\n{}{}❌ Some tests failed.{}", BOLD, RED, RESET);
        }
    }

    fn run_buck_test(&self, summary: &mut RunSummary) {
        let scenarios: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| self.matches_scenario_filter(s.name()))
            .collect();

        for scenario in scenarios {
            // Check resources at scenario level
            let resources_unavailable = scenario.check_resources().err();

            for test in scenario.tests() {
                if !self.matches_test_filter(test.name) {
                    continue;
                }

                let full_name = format!("{}::{}", scenario.name(), test.name);

                // Output JSON start event
                println!("{{\"op\":\"start\",\"test\":\"{}\"}}", full_name);

                let status = if let Some(ref reason) = resources_unavailable {
                    println!(
                        "{{\"op\":\"test_done\",\"test\":\"{}\",\"status\":\"skipped\",\"details\":\"{}\"}}",
                        full_name,
                        escape_json(reason)
                    );
                    log_message(&format!("⏭️  SKIPPED: {} ({})", full_name, reason));
                    summary.skipped += 1;
                    TestStatus::Skipped(reason.clone())
                } else {
                    let status = self.run_test(test);
                    match &status {
                        TestStatus::Passed => {
                            println!(
                                "{{\"op\":\"test_done\",\"test\":\"{}\",\"status\":\"passed\"}}",
                                full_name
                            );
                            log_message(&format!("✓ PASSED: {}", full_name));
                            summary.passed += 1;
                        }
                        TestStatus::Failed(e) => {
                            println!(
                                "{{\"op\":\"test_done\",\"test\":\"{}\",\"status\":\"failed\",\"details\":\"{}\"}}",
                                full_name,
                                escape_json(e)
                            );
                            log_message(&format!("✗ FAILED: {} - {}", full_name, e));
                            summary.failed += 1;
                        }
                        TestStatus::Skipped(reason) => {
                            println!(
                                "{{\"op\":\"test_done\",\"test\":\"{}\",\"status\":\"skipped\",\"details\":\"{}\"}}",
                                full_name,
                                escape_json(reason)
                            );
                            log_message(&format!("⏭️  SKIPPED: {} ({})", full_name, reason));
                            summary.skipped += 1;
                        }
                    }
                    status
                };

                summary.results.push(TestRunResult {
                    scenario: scenario.name().to_string(),
                    test: test.name.to_string(),
                    status: status.clone(),
                });

                if self.config.fail_fast && status.is_failed() {
                    return;
                }
            }
        }
    }

    fn run_test(&self, test: &Test) -> TestStatus {
        // Check if test's resources are available
        match self.check_test_resources(test) {
            Some(ResourceCheckResult::Skipped(reason)) => {
                return TestStatus::Skipped(reason);
            }
            Some(ResourceCheckResult::Failed(reason)) => {
                return TestStatus::Failed(reason);
            }
            None => {}
        }

        match test.run() {
            Ok(()) => TestStatus::Passed,
            Err(e) => TestStatus::Failed(e.to_string()),
        }
    }

    /// Result of checking a test's resource availability
    fn check_test_resources(&self, test: &Test) -> Option<ResourceCheckResult> {
        if test.resources.is_empty() {
            return None;
        }

        let state_path = std::env::var("TTY_TEST_RESOURCES_FILE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from(STATE_FILE_NAME));

        let state = match ResourceState::load(&state_path) {
            Ok(state) => state,
            Err(_) => {
                return Some(ResourceCheckResult::Skipped(format!(
                    "Resource state file not found. Run with --with-setup to provision resources: {:?}",
                    test.resources
                )));
            }
        };

        for resource_id in test.resources {
            // Check if setup was attempted and failed
            if let Some(error) = state.failures.get(*resource_id) {
                return Some(ResourceCheckResult::Failed(format!(
                    "Resource '{}' setup failed: {}",
                    resource_id, error
                )));
            }
            match state.get_handle::<serde_json::Value>(resource_id) {
                Ok(Some(_)) => {}
                Ok(None) => {
                    return Some(ResourceCheckResult::Skipped(format!(
                        "Resource '{}' not found. Run with --with-setup to provision.",
                        resource_id
                    )));
                }
                Err(e) => {
                    return Some(ResourceCheckResult::Failed(format!(
                        "Error checking resource '{}': {}",
                        resource_id, e
                    )));
                }
            }
        }

        None
    }

    /// Run and exit with appropriate code
    pub fn run_and_exit(&self) -> ! {
        let summary = self.run();
        std::process::exit(if summary.all_passed() { 0 } else { 1 })
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ResourceCheck;
    use crate::resources::TestResource;
    use crate::scenario::Test;
    use crate::scenario::TestResult;

    struct AlwaysUnavailableResource {
        reason: String,
    }

    impl AlwaysUnavailableResource {
        fn new(reason: &str) -> Self {
            Self {
                reason: reason.to_string(),
            }
        }
    }

    impl TestResource for AlwaysUnavailableResource {
        fn resource_type(&self) -> &'static str {
            "mock"
        }

        fn description(&self) -> String {
            "Mock unavailable resource".to_string()
        }

        fn check(&self) -> ResourceCheck {
            ResourceCheck::unavailable(&self.reason)
        }
    }

    struct AlwaysAvailableResource;

    impl TestResource for AlwaysAvailableResource {
        fn resource_type(&self) -> &'static str {
            "mock"
        }

        fn description(&self) -> String {
            "Mock available resource".to_string()
        }

        fn check(&self) -> ResourceCheck {
            ResourceCheck::available()
        }
    }

    fn passing_test() -> TestResult {
        Ok(())
    }

    fn failing_test() -> TestResult {
        Err("intentional failure".into())
    }

    #[test]
    fn test_skip_when_resource_unavailable() {
        let scenario = Scenario::new("test_scenario")
            .with_resource(Box::new(AlwaysUnavailableResource::new(
                "resource not available",
            )))
            .with_test(Test::new("should_be_skipped", passing_test));

        let mut runner = ScenarioRunner::standalone();
        runner.add_scenario(scenario);

        let summary = runner.run();

        assert_eq!(summary.skipped, 1, "expected 1 test to be skipped");
        assert_eq!(summary.passed, 0, "expected 0 tests to pass");
        assert_eq!(summary.failed, 0, "expected 0 tests to fail");

        let result = &summary.results[0];
        assert!(
            matches!(&result.status, TestStatus::Skipped(reason) if reason.contains("resource not available")),
            "expected skip reason to contain 'resource not available'"
        );
    }

    #[test]
    fn test_run_when_resource_available() {
        let scenario = Scenario::new("test_scenario")
            .with_resource(Box::new(AlwaysAvailableResource))
            .with_test(Test::new("should_pass", passing_test));

        let mut runner = ScenarioRunner::standalone();
        runner.add_scenario(scenario);

        let summary = runner.run();

        assert_eq!(summary.passed, 1, "expected 1 test to pass");
        assert_eq!(summary.skipped, 0, "expected 0 tests to be skipped");
        assert_eq!(summary.failed, 0, "expected 0 tests to fail");
    }

    #[test]
    fn test_skip_all_tests_in_scenario_when_resource_unavailable() {
        let scenario = Scenario::new("test_scenario")
            .with_resource(Box::new(AlwaysUnavailableResource::new(
                "missing dependency",
            )))
            .with_test(Test::new("test_one", passing_test))
            .with_test(Test::new("test_two", passing_test))
            .with_test(Test::new("test_three", failing_test));

        let mut runner = ScenarioRunner::standalone();
        runner.add_scenario(scenario);

        let summary = runner.run();

        assert_eq!(
            summary.skipped, 3,
            "expected all 3 tests to be skipped when resource is unavailable"
        );
        assert_eq!(summary.passed, 0, "expected 0 tests to pass");
        assert_eq!(summary.failed, 0, "expected 0 tests to fail");

        for result in &summary.results {
            assert!(
                matches!(&result.status, TestStatus::Skipped(reason) if reason.contains("missing dependency")),
                "each skipped test should have the resource unavailable reason"
            );
        }
    }

    #[test]
    fn test_fail_when_resource_setup_failed() {
        use crate::resources::ResourceState;
        use crate::resources::STATE_FILE_NAME;

        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join(STATE_FILE_NAME);

        // Create a state file where "mast:broken" has a recorded failure
        let mut state = ResourceState::default();
        state
            .failures
            .insert("mast:broken".to_string(), "connection refused".to_string());
        state.save(&state_path).unwrap();

        // Point the runner to our state file
        // SAFETY: Test runs in isolation
        unsafe {
            std::env::set_var("TTY_TEST_RESOURCES_FILE", state_path.to_str().unwrap());
        }

        let scenario = Scenario::new("test_scenario")
            .with_resource(Box::new(AlwaysAvailableResource))
            .with_test(Test::with_resources(
                "should_fail_not_skip",
                passing_test,
                &["mast:broken"],
            ));

        let mut runner = ScenarioRunner::standalone();
        runner.add_scenario(scenario);

        let summary = runner.run();

        // Cleanup env var before assertions
        // SAFETY: Restoring environment
        unsafe {
            std::env::remove_var("TTY_TEST_RESOURCES_FILE");
        }

        assert_eq!(summary.failed, 1, "expected 1 test to FAIL (not skip)");
        assert_eq!(summary.skipped, 0, "expected 0 tests to be skipped");
        assert_eq!(summary.passed, 0, "expected 0 tests to pass");

        let result = &summary.results[0];
        assert!(
            matches!(&result.status, TestStatus::Failed(reason) if reason.contains("setup failed")),
            "expected failure reason to contain 'setup failed', got: {:?}",
            result.status
        );
    }

    #[test]
    fn test_skip_when_resource_not_provisioned() {
        use crate::resources::ResourceState;
        use crate::resources::STATE_FILE_NAME;

        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join(STATE_FILE_NAME);

        // Create a state file with no entry for the required resource
        // (resource was never provisioned, not a failure)
        let state = ResourceState::default();
        state.save(&state_path).unwrap();

        // SAFETY: Test runs in isolation
        unsafe {
            std::env::set_var("TTY_TEST_RESOURCES_FILE", state_path.to_str().unwrap());
        }

        let scenario = Scenario::new("test_scenario")
            .with_resource(Box::new(AlwaysAvailableResource))
            .with_test(Test::with_resources(
                "should_skip",
                passing_test,
                &["mast:never_provisioned"],
            ));

        let mut runner = ScenarioRunner::standalone();
        runner.add_scenario(scenario);

        let summary = runner.run();

        // SAFETY: Restoring environment
        unsafe {
            std::env::remove_var("TTY_TEST_RESOURCES_FILE");
        }

        assert_eq!(summary.skipped, 1, "expected 1 test to be skipped");
        assert_eq!(summary.failed, 0, "expected 0 tests to fail");
        assert_eq!(summary.passed, 0, "expected 0 tests to pass");

        let result = &summary.results[0];
        assert!(
            matches!(&result.status, TestStatus::Skipped(reason) if reason.contains("not found")),
            "expected skip reason to contain 'not found', got: {:?}",
            result.status
        );
    }

    #[test]
    fn test_multiple_scenarios_independent_resource_checks() {
        let scenario_with_unavailable = Scenario::new("unavailable_scenario")
            .with_resource(Box::new(AlwaysUnavailableResource::new("not found")))
            .with_test(Test::new("skipped_test", passing_test));

        let scenario_with_available = Scenario::new("available_scenario")
            .with_resource(Box::new(AlwaysAvailableResource))
            .with_test(Test::new("running_test", passing_test));

        let mut runner = ScenarioRunner::standalone();
        runner.add_scenario(scenario_with_unavailable);
        runner.add_scenario(scenario_with_available);

        let summary = runner.run();

        assert_eq!(summary.skipped, 1, "expected 1 test to be skipped");
        assert_eq!(summary.passed, 1, "expected 1 test to pass");
        assert_eq!(summary.failed, 0, "expected 0 tests to fail");
    }
}
