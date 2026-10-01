/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Scenario-based test framework
//!
//! A Scenario groups related tests and their resource requirements.
//! This is a generic test organization framework that can be used for any
//! CLI application testing.

use crate::prelude::TestResource;

/// Result of running a test
pub type TestResult = Result<(), TestError>;

/// A test function that can be run by a scenario
pub type TestFn = fn() -> TestResult;

/// Error type for test failures
#[derive(Debug)]
pub struct TestError {
    pub message: String,
    pub source: Option<anyhow::Error>,
}

impl TestError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn from_anyhow(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
            source: Some(err),
        }
    }
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

impl From<anyhow::Error> for TestError {
    fn from(err: anyhow::Error) -> Self {
        Self::from_anyhow(err)
    }
}

impl From<&str> for TestError {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for TestError {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<regex::Error> for TestError {
    fn from(err: regex::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<std::io::Error> for TestError {
    fn from(err: std::io::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for TestError {
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::new(err.to_string())
    }
}

impl From<std::env::VarError> for TestError {
    fn from(err: std::env::VarError) -> Self {
        Self::new(err.to_string())
    }
}

/// A named test within a scenario
pub struct Test {
    /// Name of the test
    pub name: &'static str,
    /// The test function
    pub test_fn: TestFn,
    /// Resource IDs required by this test
    pub resources: &'static [&'static str],
}

impl Test {
    pub const fn new(name: &'static str, test_fn: TestFn) -> Self {
        Self {
            name,
            test_fn,
            resources: &[],
        }
    }

    /// Create a new test with resource requirements
    pub const fn with_resources(
        name: &'static str,
        test_fn: TestFn,
        resources: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            test_fn,
            resources,
        }
    }

    pub fn run(&self) -> TestResult {
        (self.test_fn)()
    }
}

/// A setup requirement for a scenario.
///
/// Setup requirements are external dependencies that must be prepared
/// before tests can run. They are categorized by kind (e.g., "service",
/// "database", "config") and have a value identifying the specific requirement.
///
/// The test runner itself doesn't interpret these - they're passed to
/// external setup tools that understand the specific requirement kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupRequirement {
    /// The kind/category of requirement (e.g., "service", "database", "config")
    pub kind: &'static str,
    /// The specific requirement value (e.g., "my-service-config")
    pub value: &'static str,
}

impl SetupRequirement {
    pub const fn new(kind: &'static str, value: &'static str) -> Self {
        Self { kind, value }
    }
}

/// A Scenario groups related tests with shared resource requirements.
///
/// Scenarios are the primary unit of test organization. Each scenario:
/// - Has a unique name for filtering and reporting
/// - Declares setup requirements (external dependencies for tooling)
/// - Contains one or more test functions
///
/// # Example
///
/// ```rust,ignore
/// fn my_test() -> TestResult {
///     // test code
///     Ok(())
/// }
///
/// pub fn scenario() -> Scenario {
///     Scenario::new("my_scenario")
///         .with_test(Test::new("my_test", my_test))
///         .with_setup_requirement(SetupRequirement::new("service", "my-service"))
/// }
/// ```
pub struct Scenario {
    /// Unique name of the scenario
    name: String,
    /// Tests in this scenario
    tests: Vec<Test>,
    /// Runtime resources required by this scenario (checked at test time)
    resources: Vec<Box<dyn TestResource>>,
    /// Setup requirements for external tooling (e.g., job launchers)
    setup_requirements: Vec<SetupRequirement>,
}

impl Scenario {
    /// Create a new scenario with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tests: Vec::new(),
            resources: Vec::new(),
            setup_requirements: Vec::new(),
        }
    }

    /// Add a test to this scenario
    pub fn with_test(mut self, test: Test) -> Self {
        self.tests.push(test);
        self
    }

    /// Add a runtime resource requirement (checked before test execution)
    pub fn with_resource(mut self, resource: Box<dyn TestResource>) -> Self {
        self.resources.push(resource);
        self
    }

    /// Add a setup requirement for external tooling
    pub fn with_setup_requirement(mut self, requirement: SetupRequirement) -> Self {
        self.setup_requirements.push(requirement);
        self
    }

    /// Get the scenario name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the tests in this scenario
    pub fn tests(&self) -> &[Test] {
        &self.tests
    }

    /// Get the setup requirements for this scenario
    pub fn setup_requirements(&self) -> &[SetupRequirement] {
        &self.setup_requirements
    }

    /// Check if all runtime resources for this scenario are available
    pub fn check_resources(&self) -> Result<(), String> {
        for resource in &self.resources {
            if !resource.is_available() {
                return Err(format!(
                    "{} unavailable: {}",
                    resource.resource_type(),
                    resource.unavailable_reason().unwrap_or_default()
                ));
            }
        }
        Ok(())
    }

    /// Check if runtime resources are available (returns bool for simpler checks)
    pub fn resources_available(&self) -> bool {
        self.check_resources().is_ok()
    }
}

/// Collect all unique setup requirements of a specific kind from a list of scenarios
/// Returns the values (not references) for the given kind
pub fn collect_setup_requirements_by_kind(scenarios: &[Scenario], kind: &str) -> Vec<&'static str> {
    let mut values: Vec<&'static str> = Vec::new();
    for scenario in scenarios {
        for req in scenario.setup_requirements() {
            if req.kind == kind && !values.contains(&req.value) {
                values.push(req.value);
            }
        }
    }
    values
}
