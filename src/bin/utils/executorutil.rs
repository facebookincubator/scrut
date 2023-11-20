use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use scrut::executors::bash_runner::BashRunner;
use scrut::executors::bash_script_executor::BashScriptExecutor;
use scrut::executors::executor::Executor;
use scrut::executors::stateful_executor::StatefulExecutor;

use super::environment::canonical_shell;

pub(crate) fn make_executor(
    shell: &Path,
    timeout_seconds: u64,
    cram_compat: bool,
) -> Result<(Option<Duration>, Box<dyn Executor>)> {
    let shell = canonical_shell(shell)?;
    Ok((
        if timeout_seconds > 0 {
            Some(Duration::from_secs(timeout_seconds))
        } else {
            None
        },
        if cram_compat {
            Box::new(BashScriptExecutor::new(&shell))
        } else {
            Box::new(StatefulExecutor::new(BashRunner::stateful_generator(
                &shell,
            )))
        },
    ))
}
