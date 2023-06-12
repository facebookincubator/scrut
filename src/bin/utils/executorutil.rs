use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use scrut::executors::executor::Executor;
use scrut::executors::sequential_executor::SequentialShellExecutor;

use super::fsutil::canonical_output_path;

pub(crate) fn make_executor(
    shell: &str,
    timeout_seconds: usize,
) -> Result<(Option<Duration>, Box<dyn Executor>)> {
    let shell = canonical_shell(shell)?;
    Ok((
        if timeout_seconds > 0 {
            Some(Duration::from_secs(timeout_seconds as u64))
        } else {
            None
        },
        Box::new(SequentialShellExecutor::new(&shell)),
    ))
}

/// Returns the canonical path to the given shell
pub(crate) fn canonical_shell(shell: &str) -> Result<String> {
    let path = Path::new(shell);
    if path.components().count() > 1 {
        canonical_output_path(path)
    } else {
        canonical_output_path(which::which(shell).context("guessing path to shell")?)
    }
    .context("path to shell")
}
