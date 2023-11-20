use std::ops::Add;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use tempfile::TempDir;
use tracing::trace;
use tracing::trace_span;

use super::context::Context as ExecutionContext;
use super::error::ExecutionError;
use super::executor::Executor;
use super::executor::Result;
use super::executor::DEFAULT_TOTAL_TIMEOUT;
use super::runner::Runner;
use crate::config::DEFAULT_SKIP_DOCUMENT_CODE;
use crate::executors::error::ExecutionTimeout;
use crate::output::ExitStatus;
use crate::output::Output;
use crate::testcase::TestCase;

/// A generator that creates a new instance of a [`super::runner::Runner`] that is provided with a
/// shared directory.
pub type StatefulExecutorRunnerGenerator = Box<dyn Fn(&Path) -> Box<dyn Runner>>;

/// An executor that maintains state between the executions by providing the [`Runner`] with a
/// shared directory, which it then can use to store and restore state (e.g.
/// [`super::bash_runner::BashRunner`]).
///
/// Each execution runs in a new shell instance and is provided a shared directory that can be used
/// to share state between the executions.
///
/// The executors supports both timeouts per executions and timeouts over all executions.
pub struct StatefulExecutor(StatefulExecutorRunnerGenerator);

/// A dataset to differentiate between occurance of global and per-execution timeout
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Timeout {
    is_global: bool,
    timeout: Duration,
}

impl StatefulExecutor {
    pub fn new(generator: StatefulExecutorRunnerGenerator) -> Self {
        Self(generator)
    }
}

impl Executor for StatefulExecutor {
    /// Run all Executions in given order. Timeout over all Executions is supported. Timeout per
    /// Execution is not.
    fn execute_all(
        &self,
        testcases: &[&TestCase],
        context: &ExecutionContext,
    ) -> Result<Vec<Output>> {
        // a temporary directory, that will be used to copy state in between the executions
        let state_directory = context
            .temp_directory
            .as_ref()
            .map_or_else(
                || TempDir::with_prefix(".state."),
                |dir| TempDir::with_prefix_in(".state.", dir),
            )
            .context("generate temporary output directory")
            .map_err(|err| ExecutionError::aborted(err, None))?;

        // prepare "global" timeout, if there is any
        let timeout_duration = context
            .config
            .total_timeout
            .unwrap_or(*DEFAULT_TOTAL_TIMEOUT);
        let timeout_at = if timeout_duration.is_zero() {
            None
        } else {
            Some(Instant::now().add(timeout_duration))
        };
        let timeout_left = || timeout_at.map(|at| at.duration_since(Instant::now()));
        let runner_gen = &self.0;

        // iterate all executions and run them in a bash process, then run
        // the next execution using the state of the previous
        let mut outputs = vec![];
        for (index, testcase) in testcases.iter().enumerate() {
            let name = format!("exec{}", index + 1);

            // timeout is whatever the lowest provided value of:
            // - global (over all executions) timeout
            // - local (per execution) timeout
            let (is_global_timeout, timeout) = vec![
                testcase.config.timeout.map(|d| Timeout {
                    is_global: false,
                    timeout: d,
                }),
                timeout_left().map(|d| Timeout {
                    is_global: true,
                    timeout: d,
                }),
            ]
            .into_iter()
            .filter(|item| item.is_some())
            .min()
            .unwrap_or_default()
            .map_or((false, None), |t| (t.is_global, Some(t.timeout)));
            let span = trace_span!("execution", expression = &testcase.shell_expression, timeout = ?&timeout);
            let _enter = span.enter();

            // execute the execution, using the shared state directory
            let context = context.to_owned();
            let mut testcase = (*testcase).clone();
            testcase.config.timeout = timeout;
            let output = runner_gen(state_directory.path())
                .run(&name, &testcase, &context)
                .map_err(|err| ExecutionError::failed(index, err))?;
            trace!("{output:?}");

            // handle exit code
            let skip_document_code = testcase
                .config
                .skip_document_code
                .unwrap_or(DEFAULT_SKIP_DOCUMENT_CODE);
            match output.exit_code {
                // having an actual numeric exit code ..
                ExitStatus::Code(code) => {
                    // .. ends collecting if user signals to skip
                    if code == skip_document_code {
                        return Err(ExecutionError::Skipped(index));
                    }

                    // .. otherwise keep collecting output
                    outputs.push(output);
                }

                // running into a timeout ends all execution ..
                ExitStatus::Timeout(_) => {
                    // .. of the whole context? (global, timeout over all executions)
                    if is_global_timeout {
                        return Err(ExecutionError::Timeout(ExecutionTimeout::Total));
                    }

                    // .. or of only this particular execution
                    return Err(ExecutionError::Timeout(ExecutionTimeout::Index(index)));
                }

                // user triggered skip ends all execution
                ExitStatus::Skipped => {
                    return Err(ExecutionError::Skipped(index));
                }

                // user says the process is running detached and we should ignore it
                ExitStatus::Detached => {
                    // nothing to do, we just ignore it
                }

                // undefined: things are hairy, better end
                ExitStatus::Unknown => {
                    outputs.push(output);
                    outputs.extend((0..(testcases.len() - outputs.len())).map(|_| Output {
                        stderr: vec![].into(),
                        stdout: vec![].into(),
                        exit_code: ExitStatus::Unknown,
                    }));
                    break;
                }
            }
        }

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use regex::Regex;

    use super::StatefulExecutor;
    use crate::executors::bash_runner::BashRunner;
    use crate::executors::context::Context as ExecutionContext;
    use crate::executors::error::ExecutionError;
    use crate::executors::error::ExecutionTimeout;
    use crate::executors::executor::tests::combined_output_test_suite;
    use crate::executors::executor::tests::run_executor_tests;
    use crate::executors::executor::tests::standard_output_test_suite;
    use crate::executors::DEFAULT_SHELL;
    use crate::output::ExitStatus;
    use crate::testcase::TestCase;

    #[test]
    fn test_standard_test_suite() {
        standard_output_test_suite(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            &ExecutionContext::default(),
        );
    }

    #[test]
    fn test_combined_output_test_suite() {
        combined_output_test_suite(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            &ExecutionContext::default(),
        );
    }

    #[test]
    fn test_executor_respects_timeout() {
        let tests = vec![
            (
                "Total timeout when exceeding single execution",
                vec![
                    TestCase::from_expression("sleep 1.0 && echo OK1"),
                    TestCase::from_expression("sleep 1.0 && echo OK2"),
                    TestCase::from_expression("sleep 1.0 && echo OK3"),
                ],
                Some(Duration::from_millis(150)),
                Err(ExecutionError::Timeout(ExecutionTimeout::Total)),
            ),
            (
                "Total timeout when exceeding per-execution",
                vec![
                    TestCase::from_expression("sleep 0.5 && echo OK1"),
                    TestCase::from_expression_timed(
                        "sleep 0.5 && echo OK2",
                        Some(Duration::from_millis(300)),
                    ),
                    TestCase::from_expression("sleep 0.5 && echo OK3"),
                ],
                Some(Duration::from_millis(1200)),
                Err(ExecutionError::Timeout(ExecutionTimeout::Index(1))),
            ),
            (
                "Total timeout exceed accumulative",
                vec![
                    TestCase::from_expression("sleep 0.1 && echo OK1"),
                    TestCase::from_expression("sleep 0.1 && echo OK2"),
                    TestCase::from_expression("sleep 0.1 && echo OK3"),
                ],
                Some(Duration::from_millis(150)),
                Err(ExecutionError::Timeout(ExecutionTimeout::Total)),
            ),
            (
                "Execution within timeout",
                vec![
                    TestCase::from_expression("sleep 0.1 && echo OK1"),
                    TestCase::from_expression("sleep 0.1 && echo OK2"),
                    TestCase::from_expression("sleep 0.1 && echo OK3"),
                ],
                // windows execution takes a long time to start up, test intends
                // to assert that timeout > actual execution does not return
                // a timeout error -> long timeout is fine
                Some(Duration::from_millis(if cfg!(windows) {
                    2000
                } else {
                    1000
                })),
                Ok(vec![
                    ("OK1\n", "").into(),
                    ("OK2\n", "").into(),
                    ("OK3\n", "").into(),
                ]),
            ),
        ];

        run_executor_tests(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            tests,
            &ExecutionContext::default(),
        );
    }

    #[test]
    fn test_supports_timeout_per_execution() {
        let tests = vec![
            (
                "Sufficient timeout has no effect",
                vec![TestCase::from_expression_timed(
                    "sleep 0.1 && echo OK1",
                    Some(Duration::from_millis(2000)),
                )],
                None,
                Ok(vec![("OK1\n", "").into()]),
            ),
            (
                "Insufficient timeout aborts execution",
                vec![TestCase::from_expression_timed(
                    "sleep 0.2 && echo OK1",
                    Some(Duration::from_millis(50)),
                )],
                None,
                Err(ExecutionError::Timeout(ExecutionTimeout::Index(0))),
            ),
            (
                "Timeout affects execution in isolation",
                vec![
                    TestCase::from_expression_timed(
                        "sleep 0.1 && echo OK1",
                        Some(Duration::from_millis(2000)),
                    ),
                    TestCase::from_expression_timed(
                        "sleep 0.1 && echo OK2",
                        Some(Duration::from_millis(10)),
                    ),
                    TestCase::from_expression_timed(
                        "sleep 0.1 && echo OK3",
                        Some(Duration::from_millis(10)),
                    ),
                    TestCase::from_expression_timed(
                        "sleep 0.1 && echo OK4",
                        Some(Duration::from_millis(2000)),
                    ),
                ],
                None,
                Err(ExecutionError::Timeout(ExecutionTimeout::Index(1))),
            ),
        ];

        run_executor_tests(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            tests,
            &ExecutionContext::default(),
        );
    }

    #[test]
    fn test_skipped_test_returns_skipped_error() {
        let tests = vec![(
            "Skip ends execution",
            vec![
                TestCase::from_expression("echo OK1"),
                TestCase::from_expression("exit 80"),
                TestCase::from_expression("echo OK2"),
            ],
            None,
            Err(ExecutionError::Skipped(1)),
        )];

        run_executor_tests(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            tests,
            &ExecutionContext::default(),
        );
    }

    #[test]
    fn test_executor_keeps_state() {
        let tests = vec![
            (
                "Environment variable persists",
                vec![
                    TestCase::from_expression("export FOO=bar"),
                    TestCase::from_expression("echo FOO=${FOO:-undefined}"),
                    TestCase::from_expression("unset FOO"),
                    TestCase::from_expression("echo FOO=${FOO:-undefined}"),
                ],
                None,
                Ok(vec![
                    ("", "").into(),
                    ("FOO=bar\n", "").into(),
                    ("", "").into(),
                    ("FOO=undefined\n", "").into(),
                ]),
            ),
            (
                "Shell variable persists",
                vec![
                    TestCase::from_expression("BAR=foo"),
                    TestCase::from_expression("echo BAR=${BAR:-undefined}"),
                    TestCase::from_expression("unset BAR"),
                    TestCase::from_expression("echo BAR=${BAR:-undefined}"),
                ],
                None,
                Ok(vec![
                    ("", "").into(),
                    ("BAR=foo\n", "").into(),
                    ("", "").into(),
                    ("BAR=undefined\n", "").into(),
                ]),
            ),
            (
                "Alias persists",
                vec![
                    TestCase::from_expression("alias foo='echo BAR'"),
                    TestCase::from_expression("foo"),
                    TestCase::from_expression("unalias foo"),
                    TestCase::from_expression("foo"),
                ],
                None,
                Ok(vec![
                    ("", "").into(),
                    ("BAR\n", "").into(),
                    ("", "").into(),
                    (
                        None,
                        Some(
                            Regex::new(": line \\d+: foo: command not found")
                                .expect("compile command not found regex"),
                        ),
                        Some(ExitStatus::Code(127)),
                    )
                        .into(),
                ]),
            ),
        ];

        run_executor_tests(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            tests,
            &ExecutionContext::default(),
        );
    }

    #[test]
    fn test_non_printable_ascii_in_output() {
        let tests = vec![(
            "Skip ends execution",
            vec![
                TestCase::from_expression("echo \"😊🦀\""),
                TestCase::from_expression("echo -e \"A\r\nB\""),
                TestCase::from_expression("echo \"🦀😊\" >&2"),
            ],
            None,
            Ok(vec![
                ("😊🦀\n", "").into(),
                ("A\nB\n", "").into(),
                ("", "🦀😊\n").into(),
            ]),
        )];

        run_executor_tests(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            tests,
            &ExecutionContext::default(),
        );
    }
}
