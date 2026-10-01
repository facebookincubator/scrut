/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Command-line interface for the scenario runner
//!
//! This module provides a generic main function that can be used to run scenarios.
//! It handles argument parsing, resource checking, listing, and test execution.
//!
//! # Usage
//!
//! In your main.rs, simply call `run_main` with your scenarios:
//!
//! ```rust,ignore
//! use scrutty::cli::run_main;
//!
//! mod scenarios;
//!
//! fn main() {
//!     run_main(scenarios::all_scenarios());
//! }
//! ```

use std::env;
use std::process;

use clap::Parser;

use crate::runner::RunnerConfig;
use crate::runner::ScenarioRunner;
use crate::scenario::Scenario;

/// TTY Test Runner - Run interactive terminal test scenarios
///
/// A framework for testing terminal-based applications with TTY interaction support.
#[derive(Debug, Default, Parser)]
#[command(name = "scenario_runner")]
#[command(about = "Run interactive terminal test scenarios")]
#[command(
    after_help = "Environment:\n  BUCK_TEST_MODE=1     Run in Buck test mode (JSON output) - alternative to --buck-test\n\nExamples:\n  runner                              Run all scenarios\n  runner shell sync pip               Run scenarios matching 'shell', 'sync', or 'pip'\n  runner --list                       List all scenarios and tests\n  runner --scenario sync              Run only scenarios matching 'sync'\n  runner --with-setup shell sync      Setup resources, run matching tests, teardown"
)]
pub struct CliOptions {
    /// List all scenarios and tests
    #[arg(short, long)]
    pub list: bool,

    /// Check if resources are available for all scenarios
    #[arg(long)]
    pub check_resources: bool,

    /// Write the snapshots this run produces instead of comparing them
    ///
    /// A snapshot test fails by design when the screen changes; this is how the
    /// author records the new one so it can be reviewed as a diff.
    #[arg(long)]
    pub update_snapshots: bool,

    /// Run only scenarios matching the given name (partial match, repeatable)
    #[arg(long = "scenario", value_name = "NAME")]
    pub scenario_filters: Vec<String>,

    /// Run only tests matching the given pattern (partial match)
    #[arg(long = "test", value_name = "PATTERN")]
    pub test_filter: Option<String>,

    /// Run in Buck test mode (JSON output)
    #[arg(long)]
    pub buck_test: bool,

    /// Stop on first test failure
    #[arg(short = 'f', long)]
    pub fail_fast: bool,

    // Resource lifecycle flags
    /// List required resources as JSON (for CI introspection)
    #[arg(long)]
    pub list_resources: bool,

    /// Full lifecycle: setup resources -> run tests -> teardown
    #[arg(long)]
    pub with_setup: bool,

    /// With --with-setup, skip teardown if tests fail (for debugging)
    #[arg(long)]
    pub keep_on_failure: bool,

    /// Setup resources only (no test execution)
    #[arg(long)]
    pub setup_resources: bool,

    /// Teardown resources only
    #[arg(long)]
    pub teardown_resources: bool,

    /// Scenario names to run (partial match, multiple allowed)
    #[arg(trailing_var_arg = true)]
    pub scenarios: Vec<String>,
}

impl CliOptions {
    /// Parse command-line options from arguments
    pub fn parse_args() -> Self {
        let mut opts = Self::parse();

        // Check for BUCK_TEST_MODE environment variable
        if env::var("BUCK_TEST_MODE").is_ok() {
            opts.buck_test = true;
        }

        opts
    }

    /// Parse command-line options from a vector of arguments (for testing)
    pub fn from_args_vec(args: &[String]) -> Self {
        let mut opts = Self::parse_from(args);

        // Check for BUCK_TEST_MODE environment variable
        if env::var("BUCK_TEST_MODE").is_ok() {
            opts.buck_test = true;
        }

        opts
    }

    /// Get the combined scenario filter from --scenario flags and positional args
    ///
    /// Returns None if no filters specified, or Some with comma-separated filters.
    /// Supports comma-separated values within individual arguments.
    pub fn scenario_filter(&self) -> Option<String> {
        let mut filters: Vec<String> = Vec::new();

        // Collect from --scenario flags
        for filter in &self.scenario_filters {
            for part in filter.split(',') {
                let part = part.trim();
                if !part.is_empty() && !filters.contains(&part.to_string()) {
                    filters.push(part.to_string());
                }
            }
        }

        // Collect from positional arguments
        for arg in &self.scenarios {
            for part in arg.split(',') {
                let part = part.trim();
                if !part.is_empty() && !filters.contains(&part.to_string()) {
                    filters.push(part.to_string());
                }
            }
        }

        if filters.is_empty() {
            None
        } else {
            Some(filters.join(","))
        }
    }
}

/// Check resources for all scenarios and exit with appropriate code
pub fn check_resources_all(scenarios: &[Scenario]) {
    let mut any_available = false;

    for scenario in scenarios {
        let available = scenario.resources_available();
        if available {
            any_available = true;
            eprintln!("✓ {} - resources available", scenario.name());
        } else {
            eprintln!("✗ {} - resources unavailable", scenario.name());
        }
    }

    if any_available {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

/// Output all required resources as JSON
///
/// This collects resource IDs from two sources:
/// 1. The `resources = [...]` attribute on `#[scenario_test]` macros
/// 2. Setup requirements from scenarios (for backward compatibility)
pub fn list_resources_json(scenarios: &[Scenario]) {
    use crate::registry::collect_resource_ids;

    let mut resource_ids: Vec<String> = Vec::new();

    // Collect from registered test resources (new system)
    for id in collect_resource_ids(None) {
        if !resource_ids.iter().any(|r| r == id) {
            resource_ids.push(id.to_string());
        }
    }

    // Also collect from scenario setup requirements (legacy/backward compat)
    for scenario in scenarios {
        for req in scenario.setup_requirements() {
            let resource_id = format!("{}:{}", req.kind, req.value);
            if !resource_ids.contains(&resource_id) {
                resource_ids.push(resource_id);
            }
        }
    }

    resource_ids.sort();

    let output = serde_json::json!({
        "resources": resource_ids
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// List all scenarios and their tests
pub fn list_scenarios(scenarios: &[Scenario]) {
    println!("Available scenarios:\n");

    for scenario in scenarios {
        let resources_status = if scenario.resources_available() {
            "✓"
        } else {
            "✗"
        };

        println!("{} {} scenario:", resources_status, scenario.name());

        for test in scenario.tests() {
            println!("    - {}", test.name);
        }

        let requirements = scenario.setup_requirements();
        if !requirements.is_empty() {
            println!("    Setup requirements:");
            for req in requirements {
                println!("      - {}: {}", req.kind, req.value);
            }
        }
        println!();
    }
}

/// Run scenarios with the given options
pub fn run_scenarios(scenarios: Vec<Scenario>, options: &CliOptions) {
    let scenario_filter = options.scenario_filter();

    let config = if options.buck_test {
        let mut config = RunnerConfig::buck_test();
        if let Some(ref filter) = scenario_filter {
            config = config.with_scenario_filter(filter);
        }
        if let Some(ref filter) = options.test_filter {
            config = config.with_test_filter(filter);
        }
        if options.fail_fast {
            config = config.with_fail_fast();
        }
        config
    } else {
        let mut config = RunnerConfig::standalone();
        if let Some(ref filter) = scenario_filter {
            config = config.with_scenario_filter(filter);
        }
        if let Some(ref filter) = options.test_filter {
            config = config.with_test_filter(filter);
        }
        if options.fail_fast {
            config = config.with_fail_fast();
        }
        config
    };

    let mut runner = ScenarioRunner::new(config);
    runner.add_scenarios(scenarios);
    runner.run_and_exit();
}

/// Set from the command line before any scenario runs; read by
/// [`updating_snapshots`].
static UPDATE_SNAPSHOTS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether `--update-snapshots` was passed.
///
/// A global because the flag is parsed by the runner and read inside a
/// scenario, and a scenario takes no arguments.
pub fn updating_snapshots() -> bool {
    UPDATE_SNAPSHOTS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Main entry point for scenario runners
///
/// This function handles all command-line argument parsing and dispatches
/// to the appropriate action (help, list, check-resources, or run).
///
/// # Example
///
/// ```rust,ignore
/// use scrutty::cli::run_main;
///
/// mod scenarios;
///
/// fn main() {
///     run_main(scenarios::all_scenarios());
/// }
/// ```
pub fn run_main(scenarios: Vec<Scenario>) {
    let options = CliOptions::parse_args();
    UPDATE_SNAPSHOTS.store(
        options.update_snapshots,
        std::sync::atomic::Ordering::Relaxed,
    );

    if options.check_resources {
        check_resources_all(&scenarios);
        return;
    }

    if options.list {
        list_scenarios(&scenarios);
        return;
    }

    if options.list_resources {
        list_resources_json(&scenarios);
        return;
    }

    if options.setup_resources {
        setup_resources_only();
        return;
    }

    if options.teardown_resources {
        teardown_resources_only();
        return;
    }

    if options.with_setup {
        run_with_setup(scenarios, &options);
        return;
    }

    run_scenarios(scenarios, &options);
}

/// Setup resources only (--setup-resources)
///
/// Sets up all resources declared by tests and saves handles to state file.
/// Does not run any tests.
fn setup_resources_only() {
    use crate::registry::collect_resource_ids;
    use crate::resources::ResourceManager;
    use crate::resources::create_resources;

    let resource_ids = collect_resource_ids(None);

    if resource_ids.is_empty() {
        eprintln!("No resources declared by tests.");
        return;
    }

    eprintln!("Setting up {} resource(s)...", resource_ids.len());

    let resources = create_resources(&resource_ids);

    if resources.is_empty() {
        eprintln!("Warning: No resource factories registered for declared resources.");
        eprintln!("Declared resources: {:?}", resource_ids);
        eprintln!("Register factories with register_resource_factory() before running.");
        process::exit(1);
    }

    let mut manager = ResourceManager::with_default_state_file();

    for (id, resource) in resources {
        eprintln!("  Registering: {}", id);
        manager.register(resource);
    }

    match manager.setup_all() {
        Ok(state) => {
            eprintln!("✓ Resources ready ({} handles)", state.handles.len());
            eprintln!("  State file: .tty_test_resources.json");
        }
        Err(e) => {
            eprintln!("✗ Resource setup failed: {}", e);
            process::exit(1);
        }
    }
}

/// Teardown resources only (--teardown-resources)
///
/// Tears down resources and removes state file.
fn teardown_resources_only() {
    use crate::registry::collect_resource_ids;
    use crate::resources::ResourceManager;
    use crate::resources::create_resources;

    let resource_ids = collect_resource_ids(None);
    let resources = create_resources(&resource_ids);

    if resources.is_empty() {
        // Just remove state file if no factories registered
        let state_path = std::path::PathBuf::from(crate::resources::STATE_FILE_NAME);
        if state_path.exists() {
            if let Err(e) = std::fs::remove_file(&state_path) {
                eprintln!("Warning: Failed to remove state file: {}", e);
            } else {
                eprintln!("✓ State file removed");
            }
        } else {
            eprintln!("No state file to remove.");
        }
        return;
    }

    let mut manager = ResourceManager::with_default_state_file();

    for (_id, resource) in resources {
        manager.register(resource);
    }

    eprintln!("Tearing down resources...");

    match manager.teardown_all() {
        Ok(()) => {
            eprintln!("✓ Resources cleaned up");
        }
        Err(e) => {
            eprintln!("Warning: Teardown had errors: {}", e);
        }
    }
}

/// Run with full resource lifecycle (--with-setup)
///
/// 1. Setup all declared resources
/// 2. Run tests
/// 3. Teardown resources (unless --keep-on-failure and tests failed)
fn run_with_setup(scenarios: Vec<Scenario>, options: &CliOptions) {
    use crate::registry::collect_resource_ids;
    use crate::resources::ResourceManager;
    use crate::resources::create_resources;

    let scenario_filter = options.scenario_filter();
    let resource_ids = collect_resource_ids(scenario_filter.as_deref());

    if resource_ids.is_empty() {
        eprintln!("No managed resources declared, running tests directly...");
        run_scenarios(scenarios, options);
        return;
    }

    eprintln!("Setting up {} resource(s)...", resource_ids.len());

    let resources = create_resources(&resource_ids);

    if resources.is_empty() {
        eprintln!("Warning: No resource factories registered for declared resources.");
        eprintln!("Declared resources: {:?}", resource_ids);
        eprintln!("Running tests anyway (resources may be pre-provisioned)...\n");
        run_scenarios(scenarios, options);
        return;
    }

    let mut manager = ResourceManager::with_default_state_file();

    for (id, resource) in resources {
        eprintln!("  Registering: {}", id);
        manager.register(resource);
    }

    // Setup
    match manager.setup_all() {
        Ok(state) => {
            if state.failures.is_empty() {
                eprintln!("✓ Resources ready ({} handles)\n", state.handles.len());
            } else {
                eprintln!(
                    "⚠ Resources partially ready ({} handles, {} failed)\n",
                    state.handles.len(),
                    state.failures.len()
                );
                for (id, error) in &state.failures {
                    eprintln!("  ✗ {}: {}", id, error);
                }
                eprintln!();
            }
        }
        Err(e) => {
            eprintln!("✗ Resource setup failed: {}", e);
            process::exit(1);
        }
    }

    // Run tests
    let mut config = if options.buck_test {
        RunnerConfig::buck_test()
    } else {
        RunnerConfig::standalone()
    };
    if let Some(ref filter) = scenario_filter {
        config = config.with_scenario_filter(filter);
    }
    if let Some(ref filter) = options.test_filter {
        config = config.with_test_filter(filter);
    }
    if options.fail_fast {
        config = config.with_fail_fast();
    }

    let mut runner = ScenarioRunner::new(config);
    runner.add_scenarios(scenarios);
    let summary = runner.run();

    let tests_passed = summary.all_passed();

    // Teardown (unless --keep-on-failure and tests failed)
    let should_teardown = tests_passed || !options.keep_on_failure;

    if should_teardown {
        eprintln!("\nTearing down resources...");
        if let Err(e) = manager.teardown_all() {
            eprintln!("Warning: teardown failed: {}", e);
        } else {
            eprintln!("✓ Resources cleaned up");
        }
    } else {
        eprintln!("\nKeeping resources (--keep-on-failure, tests failed)");
        eprintln!("Run --teardown-resources to clean up manually");
    }

    process::exit(if tests_passed { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_options_list_resources() {
        let args = vec!["runner".to_string(), "--list-resources".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.list_resources);
    }

    #[test]
    fn test_cli_options_with_setup() {
        let args = vec!["runner".to_string(), "--with-setup".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.with_setup);
        assert!(!opts.keep_on_failure);
    }

    #[test]
    fn test_cli_options_with_setup_keep_on_failure() {
        let args = vec![
            "runner".to_string(),
            "--with-setup".to_string(),
            "--keep-on-failure".to_string(),
        ];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.with_setup);
        assert!(opts.keep_on_failure);
    }

    #[test]
    fn test_cli_options_setup_resources_only() {
        let args = vec!["runner".to_string(), "--setup-resources".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.setup_resources);
    }

    #[test]
    fn test_cli_options_teardown_resources_only() {
        let args = vec!["runner".to_string(), "--teardown-resources".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.teardown_resources);
    }

    #[test]
    fn test_cli_options_scenario_filter_from_flag() {
        let args = vec![
            "runner".to_string(),
            "--scenario".to_string(),
            "sync".to_string(),
        ];
        let opts = CliOptions::from_args_vec(&args);
        assert_eq!(opts.scenario_filter(), Some("sync".to_string()));
    }

    #[test]
    fn test_cli_options_scenario_filter_from_positional() {
        let args = vec![
            "runner".to_string(),
            "shell".to_string(),
            "sync".to_string(),
        ];
        let opts = CliOptions::from_args_vec(&args);
        assert_eq!(opts.scenario_filter(), Some("shell,sync".to_string()));
    }

    #[test]
    fn test_cli_options_scenario_filter_comma_separated() {
        let args = vec!["runner".to_string(), "shell,sync,pip".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert_eq!(opts.scenario_filter(), Some("shell,sync,pip".to_string()));
    }

    #[test]
    fn test_cli_options_scenario_filter_combined() {
        let args = vec![
            "runner".to_string(),
            "--scenario".to_string(),
            "shell".to_string(),
            "sync".to_string(),
        ];
        let opts = CliOptions::from_args_vec(&args);
        assert_eq!(opts.scenario_filter(), Some("shell,sync".to_string()));
    }

    #[test]
    fn test_cli_options_list_short() {
        let args = vec!["runner".to_string(), "-l".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.list);
    }

    #[test]
    fn test_cli_options_fail_fast_short() {
        let args = vec!["runner".to_string(), "-f".to_string()];
        let opts = CliOptions::from_args_vec(&args);
        assert!(opts.fail_fast);
    }

    #[test]
    fn test_cli_options_test_filter() {
        let args = vec![
            "runner".to_string(),
            "--test".to_string(),
            "create".to_string(),
        ];
        let opts = CliOptions::from_args_vec(&args);
        assert_eq!(opts.test_filter, Some("create".to_string()));
    }
}
