/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::ops::Add;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use tempfile::TempDir;
use tracing::debug;
use tracing::trace;
use tracing::trace_span;

use super::context::Context as ExecutionContext;
use super::error::ExecutionError;
use super::executor::DEFAULT_TOTAL_TIMEOUT;
use super::executor::Executor;
use super::executor::Result;
use super::runner::Runner;
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
        let state_directory = TempDir::with_prefix_in(".state.", &context.temp_directory)
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
            let mut testcase = (*testcase).clone();

            // apply document-wide testcase defaults
            testcase.config = testcase.config.with_defaults_from(&context.config.defaults);

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

            // waiting on previous execution
            if let Some(ref wait) = testcase.config.wait {
                debug!("waiting {}", wait);
                if let Some(ref path) = wait.path {
                    wait_until_path_or_time(&context.temp_directory.join(path), wait.timeout)
                } else {
                    sleep(wait.timeout);
                }
            }

            // set timeout and identifying environment variable
            testcase.config.timeout = timeout;
            testcase.config.environment.insert(
                "SCRUT_TEST".into(),
                format!(
                    "{}:{}",
                    context.file.to_string_lossy(),
                    testcase.line_number
                ),
            );

            // run the execution, using the shared state directory
            let context = context.to_owned();

            trace!("effective testcase configuration: {}", &testcase.config);
            let mut output = runner_gen(state_directory.path())
                .run(&name, &testcase, context)
                .map_err(|err| ExecutionError::failed(index, err))?;
            trace!("{output:?}");

            // handle exit code
            let skip_document_code = testcase.config.get_skip_document_code();
            match output.exit_code {
                // having an actual numeric exit code ..
                ExitStatus::Code(code) => {
                    // .. ends collecting if user signals to skip
                    if code == skip_document_code {
                        return Err(ExecutionError::Skipped(index));
                    }

                    // .. otherwise keep collecting output
                    outputs.push(output);

                    // check if fail_fast is enabled and validation fails
                    if testcase.config.get_fail_fast() {
                        if testcase.validate(outputs.last().unwrap()).is_err() {
                            return Err(ExecutionError::Failed(index, outputs));
                        }
                    }
                }

                // running into a timeout ends all execution ..
                ExitStatus::Timeout(_) => {
                    // .. of the whole context? (global, timeout over all executions)
                    if is_global_timeout {
                        // ensure the original timeout is set here, because the global timeout is
                        // per all tests in one document and the per-testcase-set timeout is the
                        // remaining timeout, not the total.
                        output.exit_code = ExitStatus::Timeout(timeout_duration);
                        outputs.push(output);
                        return Err(ExecutionError::Timeout(ExecutionTimeout::Total, outputs));
                    }

                    // .. or of only this particular execution
                    outputs.push(output);
                    return Err(ExecutionError::Timeout(
                        ExecutionTimeout::Index(index),
                        outputs,
                    ));
                }

                // user triggered skip ends all execution
                ExitStatus::Skipped => {
                    return Err(ExecutionError::Skipped(index));
                }

                // user says the process is running detached and we should ignore it
                ExitStatus::Detached => outputs.push(Output {
                    exit_code: ExitStatus::Detached,
                    detached_process: output.detached_process,
                    ..Default::default()
                }),

                // undefined: things are hairy, better end
                ExitStatus::Unknown => {
                    outputs.push(output);
                    outputs.extend((0..(testcases.len() - outputs.len())).map(|_| Output {
                        exit_code: ExitStatus::Unknown,
                        ..Default::default()
                    }));
                    break;
                }
            }
        }

        Ok(outputs)
    }
}

fn wait_until_path_or_time(path: &Path, timeout: Duration) {
    let end = Instant::now().add(timeout);
    while end > Instant::now() {
        if path.exists() {
            return;
        }
        sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use regex::Regex;

    use super::StatefulExecutor;
    use crate::executors::DEFAULT_SHELL;
    use crate::executors::bash_runner::BashRunner;
    use crate::executors::error::ExecutionError;
    use crate::executors::error::ExecutionTimeout;
    use crate::executors::executor::tests::combined_output_test_suite;
    use crate::executors::executor::tests::run_executor_tests;
    use crate::executors::executor::tests::standard_output_test_suite;
    use crate::output::ExitStatus;
    use crate::output::Output;
    use crate::testcase::TestCase;

    #[test]
    fn test_standard_test_suite() {
        standard_output_test_suite(StatefulExecutor(BashRunner::stateful_generator(
            *DEFAULT_SHELL,
        )));
    }

    #[test]
    fn test_combined_output_test_suite() {
        combined_output_test_suite(StatefulExecutor(BashRunner::stateful_generator(
            *DEFAULT_SHELL,
        )));
    }

    #[test]
    fn test_executor_respects_timeout() {
        // CAREFUL: Do not reduce sleep and timeout values! Windows execution
        // comes with a significatn overhead in startup time. These values
        // are chosen to ensure the tests do not flake in Windows.
        let tests = vec![
            (
                "Execution aborted when single test-case execution time exceeds per-document timeout",
                vec![
                    TestCase::from_expression("sleep 1.0 && echo OK1"),
                    TestCase::from_expression("sleep 1.0 && echo OK2"),
                    TestCase::from_expression("sleep 1.0 && echo OK3"),
                ],
                Some(Duration::from_millis(150)),
                Err(ExecutionError::Timeout(
                    ExecutionTimeout::Total,
                    vec![Output {
                        exit_code: ExitStatus::Timeout(Duration::from_millis(150)),
                        ..Default::default()
                    }],
                )),
            ),
            (
                "Execution aborted when test-case execution time exceeds per-testcase timeout",
                vec![
                    TestCase::from_expression("sleep 0.5 && echo OK1"),
                    TestCase::from_expression_timed(
                        "sleep 0.5 && echo OK2",
                        Some(Duration::from_millis(300)),
                    ),
                    TestCase::from_expression("sleep 0.5 && echo OK3"),
                ],
                Some(Duration::from_millis(1200)),
                Err(ExecutionError::Timeout(
                    ExecutionTimeout::Index(1),
                    vec![
                        Output {
                            exit_code: ExitStatus::SUCCESS,
                            stdout: "OK1\n".into(),
                            ..Default::default()
                        },
                        Output {
                            exit_code: ExitStatus::Timeout(Duration::from_millis(300)),
                            ..Default::default()
                        },
                    ],
                )),
            ),
            (
                "Execution aborted when cumulative test-case execution time exceeds per-document timeout",
                vec![
                    TestCase::from_expression("sleep 1 && echo OK1"),
                    TestCase::from_expression("sleep 1 && echo OK2"),
                    TestCase::from_expression("sleep 1 && echo OK3"),
                ],
                Some(Duration::from_millis(1500)),
                Err(ExecutionError::Timeout(
                    ExecutionTimeout::Total,
                    vec![
                        Output {
                            exit_code: ExitStatus::SUCCESS,
                            stdout: "OK1\n".into(),
                            ..Default::default()
                        },
                        Output {
                            exit_code: ExitStatus::Timeout(Duration::from_millis(1500)),
                            ..Default::default()
                        },
                    ],
                )),
            ),
            (
                "Execution not aborted when no timeout is triggered",
                vec![
                    TestCase::from_expression("sleep 0.1 && echo OK1"),
                    TestCase::from_expression("sleep 0.1 && echo OK2"),
                    TestCase::from_expression("sleep 0.1 && echo OK3"),
                ],
                Some(Duration::from_secs(2)),
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
                Err(ExecutionError::Timeout(
                    ExecutionTimeout::Index(0),
                    vec![Output {
                        exit_code: ExitStatus::Timeout(Duration::from_millis(50)),
                        ..Default::default()
                    }],
                )),
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
                Err(ExecutionError::Timeout(
                    ExecutionTimeout::Index(1),
                    vec![
                        Output {
                            exit_code: ExitStatus::SUCCESS,
                            stdout: "OK1\n".into(),
                            ..Default::default()
                        },
                        Output {
                            exit_code: ExitStatus::Timeout(Duration::from_millis(10)),
                            ..Default::default()
                        },
                    ],
                )),
            ),
        ];

        run_executor_tests(
            StatefulExecutor(BashRunner::stateful_generator(*DEFAULT_SHELL)),
            tests,
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
        );
    }
}
