/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! This module is concerned with the execution of shell expressions from
//! [`crate::testcase::TestCase`]s. The output of such executions are stored
//! in [`crate::output::Output`] structures, that capture the STDOUT, STDERR
//! and exit code.
//!
//! The execution happens always in batches of all TestCases within a file, as
//! described in the [`crate::executors::executor::Executor`] trait.
//!
//! Currently there are two implementations available:
//! - [`crate::executors::bash_script_executor::BashScriptExecutor`]
//! - [`crate::executors::stateful_executor::StatefulExecutor`]

use std::path::Path;

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

lazy_static! {
    static ref SHELL_PATH: String = if let Ok(value) = std::env::var("SCRUT_DEFAULT_SHELL") {
        value
    } else if cfg!(target_os = "windows") {
        "bash".to_string()
    } else {
        "/bin/bash".to_string()
    };
    pub static ref DEFAULT_SHELL: &'static Path = Path::new(&*SHELL_PATH as &str);
}
