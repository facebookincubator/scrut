/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Test registry for automatic test collection
//!
//! This module provides infrastructure for collecting tests registered
//! with the `#[scenario_test]` attribute macro.

use std::collections::HashMap;
use std::path::Path;

use crate::scenario::Scenario;
use crate::scenario::Test;
use crate::scenario::TestFn;

/// A registered test with file path for automatic scenario name derivation
///
/// The scenario name is derived from the file path during collection.
/// E.g., "scenarios/sync.rs" -> "sync"
pub struct RegisteredTestFromFile {
    /// The file path (from file!() macro)
    pub file_path: &'static str,
    /// The test name
    pub name: &'static str,
    /// The test function
    pub test_fn: TestFn,
    /// Resource IDs required by this test
    pub resources: &'static [&'static str],
}

impl RegisteredTestFromFile {
    /// Create a new registered test from file (const-compatible)
    pub const fn new(
        file_path: &'static str,
        name: &'static str,
        test_fn: TestFn,
        resources: &'static [&'static str],
    ) -> Self {
        Self {
            file_path,
            name,
            test_fn,
            resources,
        }
    }

    /// Derive the scenario name from the file path
    pub fn scenario(&self) -> String {
        Path::new(self.file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(self.file_path)
            .to_string()
    }
}

// Use inventory to collect all registered tests
inventory::collect!(RegisteredTestFromFile);

/// Collect all registered tests into scenarios
///
/// This function iterates over all tests registered with `#[scenario_test]`
/// and groups them into Scenario objects by scenario name.
pub fn collect_scenarios() -> Vec<Scenario> {
    let mut scenario_map: HashMap<String, Scenario> = HashMap::new();

    // Process tests - scenario names are derived from file paths
    for registered in inventory::iter::<RegisteredTestFromFile> {
        let scenario_name = registered.scenario();
        let test = Test::with_resources(registered.name, registered.test_fn, registered.resources);

        if let Some(scenario) = scenario_map.remove(&scenario_name) {
            scenario_map.insert(scenario_name, scenario.with_test(test));
        } else {
            let new_scenario = Scenario::new(&scenario_name).with_test(test);
            scenario_map.insert(scenario_name, new_scenario);
        }
    }

    let mut scenarios: Vec<Scenario> = scenario_map.into_values().collect();

    // Sort for consistent ordering
    scenarios.sort_by(|a, b| a.name().cmp(b.name()));

    scenarios
}

/// Collect unique resource IDs from registered tests, optionally filtered by scenario
///
/// This function iterates over all tests registered with `#[scenario_test]`
/// and collects their declared resource IDs, deduplicating them.
///
/// If `scenario_filter` is provided, only resources from tests whose scenario name
/// matches the filter are collected. The filter supports comma-separated partial matches,
/// consistent with the CLI's `--scenario` and positional argument behavior.
pub fn collect_resource_ids(scenario_filter: Option<&str>) -> Vec<&'static str> {
    let mut resource_ids: Vec<&'static str> = Vec::new();

    for registered in inventory::iter::<RegisteredTestFromFile> {
        // If a filter is provided, skip tests whose scenario doesn't match
        if let Some(filter) = scenario_filter {
            let scenario_name = registered.scenario();
            let matches = filter.split(',').any(|f| scenario_name.contains(f.trim()));
            if !matches {
                continue;
            }
        }

        for resource_id in registered.resources {
            if !resource_ids.contains(resource_id) {
                resource_ids.push(*resource_id);
            }
        }
    }

    resource_ids.sort();
    resource_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::TestResult;

    fn dummy_test() -> TestResult {
        Ok(())
    }

    // Register test entries across different "scenarios" with distinct resources.
    // File paths determine scenario names: "scenarios/alpha.rs" -> "alpha", etc.
    inventory::submit! {
        RegisteredTestFromFile::new(
            "scenarios/alpha.rs",
            "test_alpha_one",
            dummy_test,
            &["resource:alpha_only"],
        )
    }

    inventory::submit! {
        RegisteredTestFromFile::new(
            "scenarios/beta.rs",
            "test_beta_one",
            dummy_test,
            &["resource:beta_only"],
        )
    }

    inventory::submit! {
        RegisteredTestFromFile::new(
            "scenarios/gamma.rs",
            "test_gamma_one",
            dummy_test,
            &["resource:shared", "resource:gamma_only"],
        )
    }

    inventory::submit! {
        RegisteredTestFromFile::new(
            "scenarios/alpha.rs",
            "test_alpha_two",
            dummy_test,
            &["resource:shared"],
        )
    }

    /// Helper: filter collected resource IDs to only our test resources
    /// (the inventory is global, so other tests in the binary may contribute entries)
    fn test_resources(ids: Vec<&str>) -> Vec<&str> {
        ids.into_iter()
            .filter(|id| id.starts_with("resource:"))
            .collect()
    }

    #[test]
    fn test_collect_resource_ids_no_filter_returns_all() {
        let ids = test_resources(collect_resource_ids(None));
        assert!(ids.contains(&"resource:alpha_only"));
        assert!(ids.contains(&"resource:beta_only"));
        assert!(ids.contains(&"resource:gamma_only"));
        assert!(ids.contains(&"resource:shared"));
    }

    #[test]
    fn test_collect_resource_ids_single_filter() {
        let ids = test_resources(collect_resource_ids(Some("alpha")));
        assert!(ids.contains(&"resource:alpha_only"));
        assert!(ids.contains(&"resource:shared")); // alpha_two uses shared
        assert!(!ids.contains(&"resource:beta_only"));
        assert!(!ids.contains(&"resource:gamma_only"));
    }

    #[test]
    fn test_collect_resource_ids_filter_excludes_unmatched() {
        let ids = test_resources(collect_resource_ids(Some("beta")));
        assert!(ids.contains(&"resource:beta_only"));
        assert!(!ids.contains(&"resource:alpha_only"));
        assert!(!ids.contains(&"resource:gamma_only"));
        assert!(!ids.contains(&"resource:shared")); // shared is on alpha and gamma, not beta
    }

    #[test]
    fn test_collect_resource_ids_comma_separated_filter() {
        let ids = test_resources(collect_resource_ids(Some("alpha,gamma")));
        assert!(ids.contains(&"resource:alpha_only"));
        assert!(ids.contains(&"resource:gamma_only"));
        assert!(ids.contains(&"resource:shared"));
        assert!(!ids.contains(&"resource:beta_only"));
    }

    #[test]
    fn test_collect_resource_ids_partial_match() {
        // "alph" should match "alpha" via contains()
        let ids = test_resources(collect_resource_ids(Some("alph")));
        assert!(ids.contains(&"resource:alpha_only"));
        assert!(!ids.contains(&"resource:beta_only"));
    }

    #[test]
    fn test_collect_resource_ids_no_match_returns_empty() {
        let ids = test_resources(collect_resource_ids(Some("nonexistent")));
        assert!(ids.is_empty());
    }

    #[test]
    fn test_collect_resource_ids_deduplicates() {
        // "alpha,gamma" both contribute "resource:shared", should appear once
        let ids = test_resources(collect_resource_ids(Some("alpha,gamma")));
        let shared_count = ids.iter().filter(|&&id| id == "resource:shared").count();
        assert_eq!(shared_count, 1);
    }
}
