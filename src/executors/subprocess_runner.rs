use std::io::ErrorKind;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use subprocess::Exec;
use subprocess::ExitStatus;
use subprocess::Redirection;
use tracing::trace;

use super::context::Context as ExecutionContext;
use super::runner::Runner;
use super::DEFAULT_SHELL;
use crate::output::ExitStatus as OutputExitStatus;
use crate::output::Output;
use crate::testcase::TestCase;

/// A runner that starts an interpreter (usually `bash`) in a sub-process and
/// writes the shell expression of a given [`crate::testcase::TestCase`] into
/// STDIN.
///
/// Constraining the max execution time is supported.
#[derive(Clone)]
pub struct SubprocessRunner(pub(super) PathBuf);

impl Runner for SubprocessRunner {
    fn run(&self, _name: &str, testcase: &TestCase, context: &ExecutionContext) -> Result<Output> {
        let shell = &self.0;

        // apply environment variables (assure SHELL is set)
        let mut envs = testcase.config.environment.clone();
        envs.insert("SHELL".into(), shell.to_string_lossy().to_string());

        let mut exec = Exec::cmd(shell)
            .stdout(Redirection::Pipe)
            // TODO(T138035235) coverage is currently using wrong libs
            .env_remove("LD_PRELOAD")
            .stderr(
                if testcase.config.output_stream
                    == Some(crate::config::OutputStreamControl::Combined)
                {
                    Redirection::Merge
                } else {
                    Redirection::Pipe
                },
            )
            .stdin(Redirection::Pipe)
            //.stdin(&execution.expression as &str)
            .env_extend(&Vec::from_iter(envs.iter()));
        if let Some(ref directory) = context.work_directory {
            exec = exec.cwd(directory);
        }

        let input = &testcase.shell_expression as &str;
        let mut process = exec.detached().popen().context("start process")?;
        let mut comm = process.communicate_start(Some(input.as_bytes().to_vec()));
        if let Some(timeout) = testcase.config.timeout {
            comm = comm.limit_time(timeout);
        }
        trace!(testcase = %&testcase, "running testcase in subprocess");

        let (stdout, stderr, exit_code) = match comm.read() {
            Ok((stdout, stderr)) => (
                stdout,
                stderr,
                process.wait().context("capture process exit")?.into(),
            ),
            Err(err) => {
                let kind = err.kind();
                let (stdout, stderr) = err.capture;

                // windows execution returns [`ErrorKind::BrokenPipe`] in case
                // anything explicitly runs `exit <code>`
                let exit = if cfg!(windows) {
                    let process_result = process.wait().unwrap_or(ExitStatus::Undetermined);
                    if kind == ErrorKind::TimedOut {
                        OutputExitStatus::Timeout(testcase.config.timeout.unwrap_or_default())
                    } else if let ExitStatus::Exited(code) = process_result {
                        (code as i32).into()
                    } else {
                        OutputExitStatus::Unknown
                    }
                } else if kind == ErrorKind::TimedOut {
                    OutputExitStatus::Timeout(testcase.config.timeout.unwrap_or_default())
                } else {
                    OutputExitStatus::Unknown
                };
                (stdout, stderr, exit)
            }
        };

        Ok(Output {
            stderr: testcase
                .render_output(&stderr.unwrap_or_default()[..])
                .to_vec()
                .into(),
            stdout: testcase
                .render_output(&stdout.unwrap_or_default()[..])
                .to_vec()
                .into(),
            exit_code,
        })
    }
}

impl Default for SubprocessRunner {
    fn default() -> Self {
        Self(DEFAULT_SHELL.to_owned())
    }
}

impl From<ExitStatus> for OutputExitStatus {
    fn from(value: ExitStatus) -> Self {
        match value {
            ExitStatus::Exited(code) => OutputExitStatus::Code(code as i32),
            ExitStatus::Signaled(_) => OutputExitStatus::Unknown,
            ExitStatus::Other(code) => OutputExitStatus::Code(code),
            ExitStatus::Undetermined => OutputExitStatus::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Runner;
    use super::SubprocessRunner;
    use crate::config::OutputStreamControl;
    use crate::config::TestCaseConfig;
    use crate::executors::context::Context as ExecutionContext;
    use crate::output::ExitStatus;
    use crate::output::Output;
    use crate::testcase::TestCase;

    #[cfg(not(target_os = "windows"))]
    #[cfg(feature = "volatile_tests")]
    #[test]
    fn test_execute_with_timeout_with_timeout() {
        let output = SubprocessRunner::default()
            .run(
                "name",
                &TestCase::from_expression("sleep 0.2 && echo OK1 && sleep 0.2 && echo OK2"),
                &ExecutionContext::default(),
            )
            .expect("execute without error");
        let expect: Output = ("OK1\nOK2\n", "").into();
        assert_eq!(expect, output);
    }

    #[test]
    fn test_execute_captures_stdout_and_stderr_separately() {
        let output = SubprocessRunner::default()
            .run(
                "name",
                &TestCase::from_expression("echo OK1 && ( 1>&2 echo OK2 )"),
                &ExecutionContext::default(),
            )
            .expect("execute without error");
        let expect: Output = ("OK1\n", "OK2\n").into();
        assert_eq!(expect, output);
    }

    #[test]
    fn test_execute_captures_stdout_and_stderr_combined() {
        let output = SubprocessRunner::default()
            .run(
                "name",
                &TestCase {
                    title: "Test".into(),
                    shell_expression: "echo OK1 && ( 1>&2 echo OK2 )".into(),
                    config: TestCaseConfig {
                        output_stream: Some(OutputStreamControl::Combined),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &ExecutionContext::default(),
            )
            .expect("execute without error");
        let expect: Output = ("OK1\nOK2\n", "").into();
        assert_eq!(expect, output);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_captures_non_printable_characters() {
        let output = SubprocessRunner::default()
            .run(
                "name",
                &TestCase::from_expression("echo -e \"🦀\r\n😊\""),
                &ExecutionContext::default(),
            )
            .expect("execute without error");

        let expect: Output = ("🦀\n😊\n", "").into();
        assert_eq!(expect, output);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_captures_exit_code() {
        let output = SubprocessRunner::default()
            .run(
                "name",
                &TestCase::from_expression("( exit 123 )"),
                &ExecutionContext::default(),
            )
            .expect("execute without error");

        let expect: Output = ("", "", Some(123)).into();
        assert_eq!(expect, output);
    }

    #[test]
    fn test_execute_respects_timeout() {
        let start = std::time::SystemTime::now();
        let output = SubprocessRunner::default()
            .run(
                "name",
                &TestCase::from_expression_timed(
                    "echo ONE && sleep 1 && echo TWO",
                    Some(Duration::from_millis(100)),
                ),
                &ExecutionContext::default(),
            )
            .expect("execution still ends in non-error");
        let duration = std::time::SystemTime::now()
            .duration_since(start)
            .expect("duration between start and now");

        assert!(
            duration >= Duration::from_millis(100),
            "waited at least 100 ms ({:?})",
            duration,
        );
        let max_wait = if cfg!(windows) { 10000 } else { 1000 };
        assert!(
            duration < Duration::from_millis(max_wait),
            "waited at most 1 s ({:?})",
            duration,
        );
        assert_eq!(
            ExitStatus::Timeout(Duration::from_millis(100)),
            output.exit_code,
            "timeout reflected in exit code",
        );
    }
}
