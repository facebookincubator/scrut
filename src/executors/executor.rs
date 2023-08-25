//! NOTE: Why not async?
//! Currently async/await is not being used, but threads are.
//!
//! The underlying argument is that concurrency is primarily used for parallel
//! execution (ParallelShellExecutor) of shell expressions, which are likely
//! magnitudes more expensive than the difference in the overhead of thread
//! spawning vs async/await. Given that the additional complication through
//! coloring and additional dependencies (tokio*) seem too steep a price to pay.
//!
//! Then again: muh.

use super::context::Context;
use super::error::ExecutionError;
use super::execution::Execution;
use crate::output::Output;

pub type Result<T> = anyhow::Result<T, ExecutionError>;

/// An Executor runs multiple [`super::execution::Execution`] at once and returns
/// their Output in the order they were provided.
///
/// Failure in execution may result in [`super::error::ExecutionError`]
pub trait Executor {
    /// Run multiple Executions and get their Output. May or may not support
    /// timeout per Execution or in total (or neither)
    fn execute_all(&self, executions: &[&Execution], context: &Context) -> Result<Vec<Output>>;
}

#[cfg(test)]
pub(super) mod tests {
    use std::time::Duration;

    use regex::Regex;

    use super::Executor;
    use super::Result;
    use crate::escaping::Escaper;
    use crate::executors::context::Context;
    use crate::executors::execution::Execution;
    use crate::output::ExitStatus;
    use crate::output::Output;

    /// A suite of tests that every executor should be able to pass
    pub(crate) fn standard_test_suite<T: Executor>(executor: T, context: &Context) {
        #[allow(clippy::type_complexity)]
        let tests: Vec<(
            &str,                        // title
            Vec<Execution>,              // input executions
            Option<Duration>,            // input duration
            Result<Vec<ExpectedOutput>>, // expected result
        )> = vec![
            (
                "STDOUT is delegated",
                vec![Execution::new("echo OK")],
                None,
                Ok(vec![("OK\n", "").into()]),
            ),
            (
                "STDERR is delegated",
                vec![Execution::new("1>&2 echo OK")],
                None,
                Ok(vec![("", "OK\n").into()]),
            ),
            (
                "Exit Code is delegated",
                vec![Execution::new("( exit 123 )")],
                None,
                Ok(vec![("", "", Some(123)).into()]),
            ),
            (
                "Multiple Executions are Delegated",
                vec![
                    Execution::new("echo OK1 && 1>&2 echo EOK1"),
                    Execution::new("echo OK2 && 1>&2 echo EOK2"),
                    Execution::new("echo OK3 && 1>&2 echo EOK3"),
                ],
                None,
                Ok(vec![
                    ("OK1\n", "EOK1\n").into(),
                    ("OK2\n", "EOK2\n").into(),
                    ("OK3\n", "EOK3\n").into(),
                ]),
            ),
            (
                "Exit code in between executions",
                vec![
                    Execution::new("echo OK1"),
                    Execution::new("( exit 123 )"),
                    Execution::new("echo OK2"),
                ],
                None,
                Ok(vec![
                    ("OK1\n", "").into(),
                    ("", "", Some(123)).into(),
                    ("OK2\n", "").into(),
                ]),
            ),
            (
                "Environment variables are set",
                vec![Execution::new("echo have $FOOBAR").environment(&[("FOOBAR", "barfoo")])],
                None,
                Ok(vec![("have barfoo\n", "").into()]),
            ),
        ];

        run_executor_tests(executor, tests, context);
    }

    /// A suite of tests that asserts executor combines output
    pub(crate) fn combined_output_test_suite<T: Executor>(executor: T, context: &Context) {
        #[allow(clippy::type_complexity)]
        let tests: Vec<(
            &str,                        // title
            Vec<Execution>,              // input executions
            Option<Duration>,            // input duration
            Result<Vec<ExpectedOutput>>, // expected result
        )> = vec![
            (
                "STDOUT is just delegated",
                vec![Execution::new("echo OK")],
                None,
                Ok(vec![("OK\n", "").into()]),
            ),
            (
                "Output in STDERR shows up in STDOUT",
                vec![Execution::new("1>&2 echo OK")],
                None,
                Ok(vec![("OK\n", "").into()]),
            ),
            (
                "Output on STDERR and STDOUT is combined to STDOUT",
                vec![Execution::new(
                    "( echo OKOUT1 ; 1>&2 echo OKERR1 ; echo OKOUT2 ; 2>&1 echo OKERR2 )",
                )],
                None,
                Ok(vec![("OKOUT1\nOKERR1\nOKOUT2\nOKERR2\n", "").into()]),
            ),
            (
                "Output on STDERR and STDOUT is combined to STDOUT",
                vec![Execution::new(
                    "( echo OKOUT1 ; 1>&2 echo OKERR1 ; echo -n OKOUT2 ; 2>&1 echo -n OKERR2 )",
                )],
                None,
                Ok(vec![("OKOUT1\nOKERR1\nOKOUT2OKERR2", "").into()]),
            ),
            (
                "Multiple execution output combines each execution's",
                vec![
                    Execution::new("( echo OKOUT1 ; 1>&2 echo OKERR1 )"),
                    Execution::new("( echo OKOUT2 ; 1>&2 echo OKERR2 )"),
                    Execution::new("( echo OKOUT3 ; 1>&2 echo OKERR3 )"),
                ],
                None,
                Ok(vec![
                    ("OKOUT1\nOKERR1\n", "").into(),
                    ("OKOUT2\nOKERR2\n", "").into(),
                    ("OKOUT3\nOKERR3\n", "").into(),
                ]),
            ),
        ];

        run_executor_tests(executor, tests, context);
    }

    /// An output expectation that can either be an expected
    /// [`crate::output::Output`] for direct comparison, or a tuple of regular
    /// expressions that must match respective STDOUT/STDERR
    pub(crate) enum ExpectedOutput {
        Output(Output),
        Regex(Option<Regex>, Option<Regex>, Option<ExitStatus>),
    }

    impl<T: ToString, U: ToString> From<(T, U, Option<i32>)> for ExpectedOutput {
        fn from(value: (T, U, Option<i32>)) -> Self {
            ExpectedOutput::Output(value.into())
        }
    }

    impl<T: ToString, U: ToString> From<(T, U)> for ExpectedOutput {
        fn from(value: (T, U)) -> Self {
            ExpectedOutput::Output(value.into())
        }
    }

    impl From<(Option<Regex>, Option<Regex>, Option<ExitStatus>)> for ExpectedOutput {
        fn from(value: (Option<Regex>, Option<Regex>, Option<ExitStatus>)) -> Self {
            ExpectedOutput::Regex(value.0, value.1, value.2)
        }
    }

    /// Encapsulates execution of table tests for executors
    #[allow(clippy::type_complexity)]
    pub(crate) fn run_executor_tests<T: Executor>(
        executor: T,
        tests: Vec<(
            &str,                        // title
            Vec<Execution>,              // input executions
            Option<Duration>,            // input duration
            Result<Vec<ExpectedOutput>>, // expected result
        )>,
        context: &Context,
    ) {
        for (title, executions, timeout, expected) in tests {
            let result = executor.execute_all(
                &executions.iter().collect::<Vec<_>>(),
                &context.with_timeout(timeout),
            );
            match expected {
                #[allow(clippy::expect_fun_call)]
                Ok(expected) => {
                    let result = result.unwrap_or_else(|err| {
                        panic!("expected success in '{}', but got: {}", title, err)
                    });
                    assert_eq!(
                        expected.len(),
                        result.len(),
                        "expected amount of outputs in '{}'",
                        title
                    );
                    for (index, expected_output) in expected.iter().enumerate() {
                        match expected_output {
                            ExpectedOutput::Output(output) => assert_eq!(
                                Some(output),
                                result.get(index),
                                "matching output in '{}' #{}",
                                title,
                                index,
                            ),
                            ExpectedOutput::Regex(stdout, stderr, exit_code) => {
                                let output = result.get(index).unwrap_or_else(|| {
                                    panic!("have output in '{}' #{}", title, index)
                                });
                                if let Some(exit_code) = exit_code {
                                    assert_eq!(
                                        &output.exit_code, exit_code,
                                        "exit status in '{}' #{}",
                                        title, index,
                                    )
                                }
                                if let Some(regex) = stdout {
                                    eprintln!("output is {:?}", output);
                                    let stdout =
                                        output.stdout.to_output_string(None, &Escaper::default());
                                    assert!(
                                        regex.is_match(&stdout),
                                        "STDOUT matches '{:?}' #{}: {:?}",
                                        regex,
                                        index,
                                        &stdout,
                                    )
                                }
                                if let Some(regex) = stderr {
                                    let stderr =
                                        output.stderr.to_output_string(None, &Escaper::default());
                                    assert!(
                                        regex.is_match(&stderr),
                                        "STDERR matches '{:?}' #{}: {:?}",
                                        regex,
                                        index,
                                        &stderr,
                                    )
                                }
                            }
                        }
                    }
                }
                #[allow(clippy::expect_fun_call)]
                Err(expected) => assert_eq!(
                    expected,
                    result.expect_err(&format!("expected failure in '{}'", title)),
                    "expected error output in '{}'",
                    title,
                ),
            }
        }
    }
}
