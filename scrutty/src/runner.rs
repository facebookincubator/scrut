/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Runner module - Scenario execution engine
//!
//! This module provides:
//! - `ScenarioRunner` - Executes scenarios and manages results
//! - `RunnerConfig` - Configuration for execution mode and filters
//! - `RunnerMode` - Standalone (stdout) vs Buck test (JSON) modes

mod runner;

pub use runner::RunSummary;
pub use runner::RunnerConfig;
pub use runner::RunnerMode;
pub use runner::ScenarioRunner;
pub use runner::TestRunResult;
pub use runner::TestStatus;

// Re-export from scenario module for convenience
pub use crate::scenario::Scenario;
pub use crate::scenario::SetupRequirement;
pub use crate::scenario::Test;
pub use crate::scenario::TestError;
pub use crate::scenario::TestFn;
pub use crate::scenario::TestResult;
pub use crate::scenario::collect_setup_requirements_by_kind;
