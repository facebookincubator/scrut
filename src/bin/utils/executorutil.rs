use std::path::Path;

use anyhow::Result;
use scrut::executors::bash_runner::BashRunner;
use scrut::executors::bash_script_executor::BashScriptExecutor;
use scrut::executors::executor::Executor;
use scrut::executors::stateful_executor::StatefulExecutor;

pub(crate) fn make_executor(shell: &Path, cram_compat: bool) -> Result<Box<dyn Executor>> {
    Ok(if cram_compat {
        Box::new(BashScriptExecutor::new(shell))
    } else {
        Box::new(StatefulExecutor::new(BashRunner::stateful_generator(shell)))
    })
}
