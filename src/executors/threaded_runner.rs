use std::io::Read;
use std::io::Write;
use std::ops::Add;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::Context;
use anyhow::Result;
use os_pipe::pipe;

use super::context::Context as ExecutionContext;
use super::execution::Execution;
use super::runner::Runner;
use super::DEFAULT_SHELL;
use crate::output::ExitStatus;
use crate::output::Output;

/// Interval in between checks for whether a shell execution has finished
const RUN_FINISHED_CHECK_INTERVAL: Duration = Duration::from_millis(5);

/// A runner that starts a (shell) process and then writes the  expression of an
/// [`crate::executors::execution::Execution`] to the STDIN from a different
/// thread.
/// Each output stream is read from an individual thread.
/// Constraining the max execution time is supported.
#[derive(Clone)]
pub struct ThreadedRunner(pub(super) String);

impl Runner for ThreadedRunner {
    fn run(
        &self,
        _name: &str,
        execution: &Execution,
        context: &ExecutionContext,
    ) -> Result<Output> {
        let shell = &self.0;
        let mut command = Command::new(shell);
        let (mut stdout_reader, stdout_writer) = pipe().context("created new STDOUT pipe")?;
        let (mut stderr_reader, stderr_writer) = if context.combine_output {
            (
                stdout_reader
                    .try_clone()
                    .context("clone STDERR reader from STDOUT")?,
                stdout_writer
                    .try_clone()
                    .context("clone STDERR writer from STDOUT")?,
            )
        } else {
            pipe().context("created new STDERR pipe")?
        };

        // initialize command
        command
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(stdout_writer)
            .stderr(stderr_writer);
        if let Some(ref directory) = context.directory {
            command.current_dir(directory);
        }

        // apply environment variables (assure SHELL is set)
        let mut envs = execution.environment.as_ref().cloned().unwrap_or_default();
        envs.insert("SHELL".into(), shell.into());

        command.envs(envs);

        #[cfg(test)]
        // TODO(T138035235) coverage is currently using wrong libs
        command.env_remove("LD_PRELOAD");

        // spawn command in a thread
        let mut child = command.spawn().with_context(|| {
            format!(
                "spawn execution in child process of shell {} in {:?}",
                shell, &context.directory,
            )
        })?;

        // pipe expression into STDIN of command (which is a shell)
        let expression = execution.expression.to_owned();
        let mut stdin = child.stdin.take().context("command STDIN")?;
        thread::spawn(move || -> anyhow::Result<()> {
            stdin
                .write_all(expression.as_bytes())
                .map_err(anyhow::Error::new)?;
            Ok(())
        });

        // start reading from STDOUT and STDERR immediately, or the (limited) pipe
        // buffers (e.g. linux defaults to 64KiB) may fill and block all writing
        let stdout_thread = thread::spawn(move || -> anyhow::Result<Vec<u8>> {
            let mut stdout = vec![];
            stdout_reader
                .read_to_end(&mut stdout)
                .context("read STDOUT from child")?;
            Ok(stdout)
        });

        let combine_output = context.combine_output;
        let stderr_thread = thread::spawn(move || -> anyhow::Result<Vec<u8>> {
            if combine_output {
                Ok(vec![])
            } else {
                let mut stderr = vec![];
                stderr_reader
                    .read_to_end(&mut stderr)
                    .context("read STDERR from child")?;
                Ok(stderr)
            }
        });

        // execute child process with or without timeout
        //eprintln!("Waiting on execution with {:?}", &execution.timeout);
        let exit_code = match &execution.timeout {
            Some(duration) => {
                // start execution in thread and wait at most for <timeout> for the execution to end
                let duration = *duration;
                let end = SystemTime::now().add(duration);
                let execution_thread = thread::spawn(move || -> Result<_> {
                    while SystemTime::now() < end {
                        // neat, exit occured before timeout
                        if let Ok(Some(status)) = child.try_wait() {
                            return Ok(match status.code() {
                                Some(code) => ExitStatus::Code(code),
                                None => ExitStatus::Unknown,
                            });
                        }
                        thread::sleep(RUN_FINISHED_CHECK_INTERVAL);
                    }

                    // dang, timeout occured, we failed to exit gracefully, we now need to cleanup
                    #[cfg(feature = "reap_on_timeout")]
                    {
                        super::reaper::reap(child.id())
                            .context("killing still running children of spawned process")?;
                    }
                    child
                        .kill()
                        .with_context(|| format!("killed child after timeout {duration:?}"))?;

                    child.wait().context("child status after timeout")?;

                    Ok(ExitStatus::Timeout(duration))
                });

                execution_thread.join().unwrap()?
            }
            None => match child.wait().context("child wait without timeout")?.code() {
                Some(code) => ExitStatus::Code(code),
                None => ExitStatus::Unknown,
            },
        };

        // drop command, so writers to (STDOUT/STDERR) pipes are closed and EOF can be reached
        drop(command);

        // wait up for STDOUT and STDERR threads to finish up
        let stdout = stdout_thread.join().unwrap()?;
        let stderr = stderr_thread.join().unwrap()?;

        Ok(Output {
            stderr: context.render_output(&stderr[..]).to_vec().into(),
            stdout: context.render_output(&stdout[..]).to_vec().into(),
            exit_code,
        })
    }
}

impl Default for ThreadedRunner {
    fn default() -> Self {
        Self(DEFAULT_SHELL.into())
    }
}

#[cfg(test)]
mod tests {
    use super::Runner;
    use super::ThreadedRunner;
    use crate::executors::context::Context as ExecutionContext;
    use crate::executors::execution::Execution;
    use crate::output::Output;

    #[cfg(not(target_os = "windows"))]
    #[cfg(feature = "volatile_tests")]
    #[test]
    fn test_execute_with_timeout_with_timeout() {
        let output = ThreadedRunner::default()
            .run(
                "name",
                &Execution::new("sleep 0.2 && echo OK1 && sleep 0.2 && echo OK2"),
                &ExecutionContext::new(),
            )
            .expect("execute without error");
        let expect: Output = ("OK1\nOK2\n", "").into();
        assert_eq!(expect, output);
    }

    #[cfg(feature = "reap_on_timeout")]
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_with_timeout_aborts_within_timeout() {
        use std::time::Duration;
        use std::time::Instant;

        use crate::escaping::Escaper;
        use crate::output::ExitStatus;

        let start = Instant::now();
        let output = ThreadedRunner::default()
            .run(
                "name",
                &Execution::new("time sleep 1 && echo OK1 && sleep 1 && echo OK2")
                    .timeout(Some(Duration::from_millis(100))),
                &ExecutionContext::default(),
            )
            .expect("no error in execution");
        let diff = start.elapsed().as_millis();
        assert_eq!(
            "".to_string(),
            output.stdout.to_output_string(None, &Escaper::default())
        );
        assert!(
            output
                .stderr
                .to_output_string(None, &Escaper::default())
                .contains("Killed: 9")
        );
        assert_eq!(
            ExitStatus::Timeout(Duration::from_millis(100)),
            output.exit_code
        );
        assert!(
            diff < 150,
            "waited for longer than timeout ({} > 150)",
            diff
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_with_timeout_captures_stdout_and_stderr() {
        let output = ThreadedRunner::default()
            .run(
                "name",
                &Execution::new("echo OK1 && ( 1>&2 echo OK2 )"),
                &ExecutionContext::new(),
            )
            .expect("execute without error");
        let expect: Output = ("OK1\n", "OK2\n").into();
        assert_eq!(expect, output);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_threaded_runner_execute_captures_non_printable_characters() {
        let output = ThreadedRunner::default()
            .run(
                "name",
                &Execution::new("echo -e \"🦀\r\n😊\""),
                &ExecutionContext::new(),
            )
            .expect("execute without error");

        let expect: Output = ("🦀\n😊\n", "").into();
        assert_eq!(expect, output);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_with_timeout_captures_exit_code() {
        let output = ThreadedRunner::default()
            .run(
                "name",
                &Execution::new("( exit 123 )"),
                &ExecutionContext::new(),
            )
            .expect("execute without error");

        let expect: Output = ("", "", Some(123)).into();
        assert_eq!(expect, output);
    }
}
