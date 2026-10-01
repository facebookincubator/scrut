/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! TTY Test Runner Framework
//!
//! A framework for writing end-to-end tests that interact with CLI applications
//! through pseudo-terminals (PTY).
//!
//! ## Architecture
//!
//! The framework is organized around **Scenarios**:
//!
//! 1. **Scenario** - Groups related tests with shared setup requirements
//! 2. **Session** - PTY session management, key input, output cleaning
//! 3. **Logging** - Structured logging with file:line, periodic session output flushing
//! 4. **Resources** - Declarative test resource requirements (files, external state)
//! 5. **Runner** - Scenario execution in standalone or buck test mode
//! 6. **Registry** - Automatic test collection via `#[scenario_test]` macro
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use scrutty::prelude::*;
//!
//! // Register test - scenario name is derived from filename
//! #[scenario_test]
//! fn my_test() -> TestResult {
//!     let session = TtySession::spawn("my-cli", &["arg"], ".", vec![])?;
//!     session.wait_for_output("prompt>", 5000, false)?;
//!     session.send_keys(&[Keys::ENTER])?;
//!     Ok(())
//! }
//!
//! // With named resource - constant is injected into function
//! #[scenario_test(resource MAST_JOB = "mast:gtn_infra")]
//! fn my_test_with_resources() -> TestResult {
//!     // MAST_JOB is available as const &str
//!     let job: MastJobHandle = resource_handle(MAST_JOB)?;
//!     Ok(())
//! }
//! ```

pub mod cli;
pub mod logging;
pub mod registry;
pub mod resources;
pub mod runner;
pub mod scenario;
pub mod session;
pub mod terminal;

// Re-export inventory for use by the proc-macro
pub use inventory;
// Re-export RegisteredTestFromFile for use by the proc-macro
pub use registry::RegisteredTestFromFile;

/// Prelude module for convenient imports
pub mod prelude {
    // Re-export the macros
    pub use scrutty_macros::scenario_test;

    // CLI
    pub use crate::cli::run_main;
    pub use crate::cli::updating_snapshots;
    // Logging
    pub use crate::logging::LogConfig;
    pub use crate::logging::SessionRegistry;
    pub use crate::logging::log_message;
    // Registry (for macro support)
    pub use crate::registry::RegisteredTestFromFile;
    pub use crate::registry::collect_resource_ids;
    pub use crate::registry::collect_scenarios;
    // Resources
    pub use crate::resources::FileResource;
    pub use crate::resources::ManagedResource;
    pub use crate::resources::ResourceCheck;
    pub use crate::resources::ResourceManager;
    pub use crate::resources::ResourceState;
    pub use crate::resources::TestResource;
    pub use crate::resources::register_resource_factory;
    pub use crate::resources::resource_handle;
    // Runner types
    pub use crate::runner::RunnerConfig;
    pub use crate::runner::RunnerMode;
    pub use crate::runner::ScenarioRunner;
    // Scenario types
    pub use crate::scenario::Scenario;
    pub use crate::scenario::SetupRequirement;
    pub use crate::scenario::Test;
    pub use crate::scenario::TestError;
    pub use crate::scenario::TestResult;
    pub use crate::scenario::collect_setup_requirements_by_kind;
    // Session
    pub use crate::session::ExpectChain;
    pub use crate::session::Keys;
    pub use crate::session::PipedOutput;
    pub use crate::session::PipedSession;
    pub use crate::session::Spawn;
    pub use crate::session::TtySession;
    pub use crate::session::clean_tty_output;
    pub use crate::session::strip_ansi_codes;
    // Terminal state
    pub use crate::terminal::Cell;
    pub use crate::terminal::Color;
    pub use crate::terminal::Menu;
    pub use crate::terminal::Style;
    pub use crate::terminal::TerminalState;
}

// Re-export commonly used types at crate root
pub use logging::SessionRegistry;
pub use logging::log_message;
pub use runner::ScenarioRunner;
pub use scenario::Scenario;
pub use scenario::SetupRequirement;
pub use scenario::Test;
pub use scenario::TestResult;
pub use session::TtySession;
