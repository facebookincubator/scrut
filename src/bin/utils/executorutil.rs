use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use scrut::executors::bash_runner::BashRunner;
use scrut::executors::bash_script_executor::BashScriptExecutor;
use scrut::executors::executor::Executor;
use scrut::executors::stateful_executor::StatefulExecutor;

use super::fsutil::canonical_path;

pub(crate) fn make_executor(
    shell: &Path,
    timeout_seconds: usize,
    cram_compat: bool,
) -> Result<(Option<Duration>, Box<dyn Executor>)> {
    let shell = canonical_shell(shell)?;
    Ok((
        if timeout_seconds > 0 {
            Some(Duration::from_secs(timeout_seconds as u64))
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

/// Returns the canonical path to the given shell
pub(crate) fn canonical_shell(shell: &Path) -> Result<PathBuf> {
    if shell.components().count() > 1 {
        canonical_path(shell)
    } else {
        canonical_path(
            which::which(shell)
                .context("guessing path to shell")?
                .as_path(),
        )
    }
    .context("path to shell")
}
