//! This module is concerned with the execution of shell expressions from
//! [`crate::testcase::TestCase`]s. The output of such executions are stored
//! in [`crate::output::Output`] structures, that capture the STDOUT, STDERR
//! and exit code.
//!
//! The execution happens always in batches of all TestCases within a file, as
//! described in the [`crate::executors::executor::Executor`] trait.
//!
//! Currently there are two implementations available:
//! - [`crate::executors::sequential_shell`], which runs executions from within
//!   a file sequentially, allowing for shared state (environment variables,
//!   aliases) to be "passed down"
//! - [`crate::executors::parallel_shell`], which runs executions from within
//!   a file in parallel, returning results quickly, but not supporting shared
//!   state in between

pub mod bash_runner;
pub mod bash_script_executor;
pub mod context;
pub mod error;
pub mod execution;
pub mod executor;
pub mod runner;
pub mod stateful_executor;
pub mod subprocess_runner;
pub mod util;

#[cfg(feature = "reap_on_timeout")]
pub mod reaper;

/// Default path to shell on Windows
#[cfg(target_os = "windows")]
pub const DEFAULT_SHELL: &str = "bash";

/// Default path to shell on Linux/Mac/..
#[cfg(not(target_os = "windows"))]
pub const DEFAULT_SHELL: &str = "/bin/bash";
