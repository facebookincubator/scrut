/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Debug;
use std::fmt::Display;

use anyhow::anyhow;

use crate::escaping::Escaper;
use crate::output::Output;

/// The state that describes what kind of timeout (i.e. per execution or over all)
/// has occured.
#[derive(Debug, PartialEq)]
pub enum ExecutionTimeout {
    /// Timeout of a specific execution (index: 0..n-1)
    Index(usize),

    /// Timeout of all executions, not a specific one
    Total,
}

/// An error that results from running [`super::executor::Executor::execute_all`],
/// which executes the shell expressions of a list of testcases.
/// ! This is not a failed validation result, or a non-zero exit code of an
/// execution, but an actual failure to execute and get a resulting Output !
#[derive(Debug, Derivative)]
#[derivative(PartialEq)]
pub enum ExecutionError {
    /// Returned if a specific [`crate::testcase::TestCase`] fails to execute.
    /// This does not mean that that test iself failed, but that something went wrong.
    FailedExecution {
        /// The index of the failed testcase as it was passed to execute_all
        index: usize,

        /// The error that prevented the execution from concluding in an Output
        #[derivative(PartialEq(compare_with = "stringable_cmp"))]
        error: anyhow::Error,
    },

    /// Returned on errors that happen during execution, but are not specific to
    /// a particular [`crate::testcase::TestCase`].
    AbortedExecutions {
        /// The cause of the execution being aborted
        #[derivative(PartialEq(compare_with = "stringable_cmp"))]
        error: anyhow::Error,

        /// Potentially the last output leading to the abort of execution
        output: Option<Output>,
    },

    /// Returned if either a single [`crate::testcase::TestCase`] execution timed
    /// out or if all are (see [`ExecutionTimeout`])
    Timeout(ExecutionTimeout, Vec<Output>),

    /// Returned if a specific [`crate::testcase::TestCase`] execution is
    /// intentionally skipped by the user.
    /// This is not a final error.
    Skipped(usize),
}

fn stringable_cmp<T: ToString>(a: T, b: T) -> bool {
    a.to_string() == b.to_string()
}

impl ExecutionError {
    /// Construct a new error without an index (e.g. when failure in execute_all)
    /// happens before or after executions take place
    pub fn aborted(error: anyhow::Error, output: Option<Output>) -> Self {
        Self::AbortedExecutions { error, output }
    }

    /// Construct a new error with an index, that denotes a specific execution
    /// from [`super::executor::Executor::execute_all`] has failed
    pub fn failed(index: usize, error: anyhow::Error) -> Self {
        Self::FailedExecution { index, error }
    }

    pub(super) fn from_execute(
        error: anyhow::Error,
        index: Option<usize>,
        output: Option<Output>,
    ) -> Self {
        match index {
            Some(index) => Self::FailedExecution { index, error },
            None => Self::AbortedExecutions { error, output },
        }
    }
}

impl<E> From<E> for ExecutionError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self::aborted(anyhow!(error), None)
    }
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::FailedExecution { index, error } => {
                write!(f, "error in execution with index={index}: {error}")
            }
            ExecutionError::AbortedExecutions { error, output } => {
                if let Some(output) = output {
                    writeln!(
                        f,
                        "aborted executions with exit code={}: {}",
                        output.exit_code, error
                    )?;
                    writeln!(f, "{}", &output.to_error_string(&Escaper::default()),)?;
                } else {
                    write!(f, "aborted executions: {error}")?;
                }
                Ok(())
            }
            ExecutionError::Timeout(timeout, _output) => match timeout {
                ExecutionTimeout::Index(idx) => write!(
                    f,
                    "timeout in executing shell expression of test {}",
                    idx + 1
                ),
                ExecutionTimeout::Total => write!(f, "timeout in executing"),
            },
            ExecutionError::Skipped(idx) => write!(f, "skipped test {}", idx + 1),
        }
    }
}
