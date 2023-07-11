use anyhow::Result;

use super::context::Context as ExecutionContext;
use super::execution::Execution;
use crate::output::Output;

/// A thing that runs a singule [`crate::executors::execution::Execution`] with
/// the given [`crate::executors::context::Context`]s.
pub trait Runner {
    /// Return the [`crate::output::Output`] of running an [`crate::executors::execution::Execution`]
    fn run(&self, name: &str, execution: &Execution, context: &ExecutionContext) -> Result<Output>;
}
