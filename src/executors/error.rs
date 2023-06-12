use std::fmt::Debug;
use std::fmt::Display;

use anyhow::anyhow;

use crate::escaping::Escaper;
use crate::output::Output;

/// An error that results from running [`super::executor::Executor::execute_all`],
/// which executes the shell expressions of a list of testcases.
/// ! This is not a failed validation result, or a non-zero exit code of an
/// execution, but an actual failure to execute and get a resulting Output !
#[derive(Debug)]
pub enum ExecutionError {
    /// Returned if a specific [`super::execution::Execution`] failed
    FailedExecution {
        /// The index of the failed execution as it was passed to execute_all
        index: usize,

        /// The error that prevented the execution from concluding in an Output
        error: anyhow::Error,
    },

    /// Returned on errors that are not specific to an [`super::execution::Execution`]
    AbortedExecutions {
        /// The cause of the execution being aborted
        error: anyhow::Error,

        /// Potentially the last output leading to the abort of execution
        output: Option<Output>,
    },

    /// Returned if either all [`super::execution::Execution`]s timed out (only
    /// when [`super::executor::Executor::execute_all`] supports timeouts)
    Timeout,

    /// Returned if any [`super::execution::Execution`] ends in [`super::SKIP_EXIT_CODE`]
    Skipped,
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

impl PartialEq for ExecutionError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::FailedExecution {
                    index: l_index,
                    error: l_error,
                },
                Self::FailedExecution {
                    index: r_index,
                    error: r_error,
                },
            ) => l_index == r_index && l_error.to_string() == r_error.to_string(),
            (
                Self::AbortedExecutions {
                    error: l_error,
                    output: l_output,
                },
                Self::AbortedExecutions {
                    error: r_error,
                    output: r_output,
                },
            ) => l_error.to_string() == r_error.to_string() && l_output == r_output,
            (Self::Timeout, Self::Timeout) => true,
            (Self::Skipped, Self::Skipped) => true,
            _ => false,
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
            ExecutionError::Timeout => write!(f, "timeout in executions"),
            ExecutionError::Skipped => write!(f, "skipped"),
        }
    }
}
