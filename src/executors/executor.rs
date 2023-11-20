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

use std::time::Duration;

use super::context::Context;
use super::error::ExecutionError;
use crate::output::Output;
use crate::testcase::TestCase;

lazy_static! {
    /// Default timeout for all executions within a single test document
    pub static ref DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(900);
}

pub type Result<T> = anyhow::Result<T, ExecutionError>;

/// An Executor runs the shell expressions of multiple [`crate::testcase::TestCase`]
/// at once and returns their Output in the order they were provided.
///
/// Failure in execution may result in [`super::error::ExecutionError`]
pub trait Executor {
    /// Run multiple Executions and get their Output. May or may not support
    /// timeout per Execution or in total (or neither)
    fn execute_all(&self, testcases: &[&TestCase], context: &Context) -> Result<Vec<Output>>;
}

#[cfg(test)]
pub(super) mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use regex::Regex;

    use super::Executor;
    use super::Result;
    use crate::config::OutputStreamControl;
    use crate::config::TestCaseConfig;
    use crate::escaping::Escaper;
    use crate::executors::context::Context;
    use crate::output::ExitStatus;
    use crate::output::Output;
    use crate::testcase::TestCase;

    /// A suite of tests that every executor should be able to pass
    pub(crate) fn standard_output_test_suite<T: Executor>(executor: T, context: &Context) {
        let create_testcase = |expr: &str| TestCase {
            title: "Test".into(),
            shell_expression: expr.into(),
            config: TestCaseConfig {
                output_stream: Some(OutputStreamControl::Stdout),
                ..Default::default()
            },
            ..Default::default()
        };

        #[allow(clippy::type_complexity)]
        let tests: Vec<(
            &str,                        // title
            Vec<TestCase>,               // input executions
            Option<Duration>,            // input duration
            Result<Vec<ExpectedOutput>>, // expected result
        )> = vec![
            (
                "STDOUT is delegated",
                vec![create_testcase("echo OK")],
                None,
                Ok(vec![("OK\n", "").into()]),
            ),
            (
                "STDERR is delegated",
                vec![create_testcase("1>&2 echo OK")],
                None,
                Ok(vec![("", "OK\n").into()]),
            ),
            (
                "Exit Code is delegated",
                vec![create_testcase("( exit 123 )")],
                None,
                Ok(vec![("", "", Some(123)).into()]),
            ),
            (
                "Multiple Executions are Delegated",
                vec![
                    create_testcase("echo OK1 && 1>&2 echo EOK1"),
                    create_testcase("echo OK2 && 1>&2 echo EOK2"),
                    create_testcase("echo OK3 && 1>&2 echo EOK3"),
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
                    create_testcase("echo OK1"),
                    create_testcase("( exit 123 )"),
                    create_testcase("echo OK2"),
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
                vec![TestCase {
                    title: "Test".into(),
                    shell_expression: "echo have $FOOBAR".into(),
                    config: TestCaseConfig {
                        environment: BTreeMap::from([("FOOBAR".into(), "barfoo".into())]),
                        output_stream: Some(OutputStreamControl::Stdout),
                        ..Default::default()
                    },
                    ..Default::default()
                }],
                None,
                Ok(vec![("have barfoo\n", "").into()]),
            ),
        ];

        run_executor_tests(executor, tests, context);
    }

    /// A suite of tests that asserts executor combines output
    pub(crate) fn combined_output_test_suite<T: Executor>(executor: T, context: &Context) {
        let create_testcase = |expr: &str| TestCase {
            title: "Test".into(),
            shell_expression: expr.into(),
            config: TestCaseConfig {
                output_stream: Some(OutputStreamControl::Combined),
                ..Default::default()
            },
            ..Default::default()
        };

        #[allow(clippy::type_complexity)]
        let tests: Vec<(
            &str,                        // title
            Vec<TestCase>,               // input executions
            Option<Duration>,            // input duration
            Result<Vec<ExpectedOutput>>, // expected result
        )> = vec![
            (
                "STDOUT is just delegated",
                vec![create_testcase("echo OK")],
                None,
                Ok(vec![("OK\n", "").into()]),
            ),
            (
                "Output in STDERR shows up in STDOUT",
                vec![create_testcase("1>&2 echo OK")],
                None,
                Ok(vec![("OK\n", "").into()]),
            ),
            (
                "Output on STDERR and STDOUT is combined to STDOUT",
                vec![create_testcase(
                    "( echo OKOUT1 ; 1>&2 echo OKERR1 ; echo OKOUT2 ; 2>&1 echo OKERR2 )",
                )],
                None,
                Ok(vec![("OKOUT1\nOKERR1\nOKOUT2\nOKERR2\n", "").into()]),
            ),
            (
                "Output on STDERR and STDOUT is combined to STDOUT",
                vec![create_testcase(
                    "( echo OKOUT1 ; 1>&2 echo OKERR1 ; echo -n OKOUT2 ; 2>&1 echo -n OKERR2 )",
                )],
                None,
                Ok(vec![("OKOUT1\nOKERR1\nOKOUT2OKERR2", "").into()]),
            ),
            (
                "Multiple execution output combines each execution's",
                vec![
                    create_testcase("( echo OKOUT1 ; 1>&2 echo OKERR1 )"),
                    create_testcase("( echo OKOUT2 ; 1>&2 echo OKERR2 )"),
                    create_testcase("( echo OKOUT3 ; 1>&2 echo OKERR3 )"),
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
            Vec<TestCase>,               // input executions
            Option<Duration>,            // input duration
            Result<Vec<ExpectedOutput>>, // expected result
        )>,
        context: &Context,
    ) {
        let total_tests = tests.len();
        for (test_index, (title, testcases, timeout, expected)) in tests.iter().enumerate() {
            let mut config = context.config.clone();
            config.total_timeout = timeout.to_owned();
            let context = Context {
                temp_directory: context.temp_directory.clone(),
                work_directory: context.work_directory.clone(),
                config,
            };
            let test_num = test_index + 1;
            let total = testcases.len();
            let result = executor.execute_all(&testcases.iter().collect::<Vec<_>>(), &context);
            match expected {
                #[allow(clippy::expect_fun_call)]
                Ok(expected) => {
                    let result = result.unwrap_or_else(|err| {
                        panic!("expected success in test #{test_num}/{total_tests} '{title}', but got: {err}")
                    });
                    assert_eq!(
                        expected.len(),
                        result.len(),
                        "expected amount of outputs in test #{test_num}/{total_tests} '{title}'",
                    );
                    for (index, expected_output) in expected.iter().enumerate() {
                        let num = index + 1;
                        match expected_output {
                            ExpectedOutput::Output(output) => assert_eq!(
                                Some(output),
                                result.get(index),
                                "matching output in test #{test_num}/{total_tests} '{title}', testcase #{num}/{total}",
                            ),
                            ExpectedOutput::Regex(stdout, stderr, exit_code) => {
                                let output = result.get(index).unwrap_or_else(|| {
                                    let count = result.len();
                                    panic!("have output in test #{test_num}/{total_tests} '{title}', testcase #{num}/{total}: found only {count} results")
                                });
                                if let Some(exit_code) = exit_code {
                                    assert_eq!(
                                        &output.exit_code, exit_code,
                                        "exit status in test #{test_num}/{total_tests} '{title}', testcase #{num}/{total}",
                                    )
                                }
                                if let Some(regex) = stdout {
                                    let stdout =
                                        output.stdout.to_output_string(None, &Escaper::default());
                                    assert!(
                                        regex.is_match(&stdout),
                                        "STDOUT matches in test #{test_num}/{total_tests} '{title}' with regex '{regex:?}', testcase #{num}/{total}: {stdout:?}",
                                    )
                                }
                                if let Some(regex) = stderr {
                                    let stderr =
                                        output.stderr.to_output_string(None, &Escaper::default());
                                    assert!(
                                        regex.is_match(&stderr),
                                        "STDERR matches in test #{test_num}/{total_tests} '{title}' with regex '{regex:?}', testcase #{num}/{total}: {stderr:?}",
                                    )
                                }
                            }
                        }
                    }
                }
                #[allow(clippy::expect_fun_call)]
                Err(expected) => assert_eq!(
                    *expected,
                    result.expect_err(&format!(
                        "expected failure in test #{test_num}/{total_tests} '{title}'"
                    )),
                    "expected error output in test #{test_num}/{total_tests} '{title}'",
                ),
            }
        }
    }
}
