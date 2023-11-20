use anyhow::Result;

use super::context::Context as ExecutionContext;
use crate::output::Output;
use crate::testcase::TestCase;

/// A thing that runs the shell expression of a single [`crate::testcase::TestCase`]
/// within the given [`crate::executors::context::Context`].
pub trait Runner {
    /// Return the [`crate::output::Output`] of running the shell expression of
    /// a [`crate::testcase::TestCase`]
    fn run(&self, name: &str, testcase: &TestCase, context: &ExecutionContext) -> Result<Output>;
}
