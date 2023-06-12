use anyhow::anyhow;
use anyhow::Context;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::ThreadPoolBuilder;

use super::context::Context as ExecutionContext;
use super::error::ExecutionError;
use super::execution::Execution;
use super::executor::Executor;
use super::executor::Result;
use super::shell::run_in_shell;
use super::util::default_parallel_count;
use super::DEFAULT_SHELL;
use crate::output::ExitStatus;
use crate::output::Output;

/// All executions take place in independent instances of the shell and therefore
/// share no state. Parallel execution is supported.
///
/// CAUTION: This executor is currently not supported! The implementation is
/// kept around for later review. Most likely it will not be further pursued.
#[derive(Clone, Debug)]
pub struct ParallelShellExecutor {
    /// Amount of parallel executions
    pub count: usize,

    /// Path to shell to use
    pub shell: String,
}

impl ParallelShellExecutor {
    /// Construct a new executor that runs `count` executions in parallel
    pub fn new(count: usize, shell: &str) -> Self {
        Self {
            count,
            shell: shell.to_owned(),
        }
    }
}

impl Default for ParallelShellExecutor {
    fn default() -> Self {
        Self {
            count: default_parallel_count(),
            shell: DEFAULT_SHELL.to_owned(),
        }
    }
}

impl Executor for ParallelShellExecutor {
    /// Run provided Executions in parallel. Timeout per execution is supported.
    /// Total timeout over all is not.
    fn execute_all(
        &self,
        executions: &[&Execution],
        context: &ExecutionContext,
    ) -> Result<Vec<Output>> {
        if context.timeout.is_some() {
            return Err(ExecutionError::aborted(
                anyhow!("parallel execution supports only timeout per execution"),
                None,
            ));
        }

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.count)
            .build()
            .context("build thread pool")
            .map_err(|err| ExecutionError::aborted(err, None))?;
        let enumerated = to_enumerated_executions(executions);
        let result = pool.install(|| {
            enumerated
                .par_iter()
                .map(|enumerated| {
                    run_in_shell(&self.shell, &enumerated.execution, context).map_err(|error| {
                        ExecutionError::from_execute(error, Some(enumerated.index), None)
                    })
                })
                .collect::<Result<Vec<_>>>()
        });

        let outputs = result?;
        if outputs
            .iter()
            .any(|output| output.exit_code == ExitStatus::SKIP)
        {
            Err(ExecutionError::Skipped)
        } else {
            Ok(outputs)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EnumeratedExecution {
    pub(crate) execution: Execution,
    pub(crate) index: usize,
}

fn to_enumerated_executions(executions: &[&Execution]) -> Vec<EnumeratedExecution> {
    executions
        .iter()
        .enumerate()
        .map(|(index, execution)| {
            let execution = (*execution).to_owned();
            EnumeratedExecution { index, execution }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::time::Instant;

    use anyhow::anyhow;

    use super::ParallelShellExecutor;
    use crate::executors::context::Context as ExecutionContext;
    use crate::executors::error::ExecutionError;
    use crate::executors::execution::Execution;
    use crate::executors::executor::tests::combined_output_test_suite;
    use crate::executors::executor::tests::run_executor_tests;
    use crate::executors::executor::tests::standard_test_suite;
    use crate::executors::executor::Executor;
    use crate::executors::util::default_parallel_count;
    use crate::executors::DEFAULT_SHELL;
    use crate::output::ExitStatus;

    #[test]
    fn test_standard_test_suite() {
        standard_test_suite(
            ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: 1,
            },
            &ExecutionContext::new(),
        );
    }

    #[test]
    fn test_combined_output_test_suite() {
        combined_output_test_suite(
            ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: 1,
            },
            &ExecutionContext::new().combine_output(true),
        );
    }

    #[test]
    fn test_executor_does_not_allow_global_timeout() {
        let tests = vec![(
            "Global timeout is not accepted",
            vec![Execution::new("sleep 0.1 && echo OK1")],
            Some(Duration::from_millis(150)),
            Err(ExecutionError::aborted(
                anyhow!("parallel execution supports only timeout per execution"),
                None,
            )),
        )];

        run_executor_tests(
            ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: 1,
            },
            tests,
            &ExecutionContext::new(),
        );
    }

    #[cfg(feature = "volatile_tests")]
    #[cfg(not(feature = "reap_on_timeout"))]
    #[test]
    fn test_supports_timeout_per_execution() {
        let tests = vec![
            (
                "Sufficient timeout has no effect",
                vec![
                    Execution::new("sleep 0.1 && echo OK1")
                        .timeout(Some(Duration::from_millis(200))),
                ],
                None,
                Ok(vec![("OK1\n", "").into()]),
            ),
            (
                "Insufficient timeout aborts execution",
                vec![
                    Execution::new("sleep 0.1 && echo OK1")
                        .timeout(Some(Duration::from_millis(50))),
                ],
                None,
                Ok(vec![Duration::from_millis(50).into()]),
            ),
            (
                "Timeout affects execution in isolation",
                vec![
                    Execution::new("sleep 0.1 && echo OK1")
                        .timeout(Some(Duration::from_millis(50))),
                    Execution::new("sleep 0.1 && echo OK2")
                        .timeout(Some(Duration::from_millis(200))),
                    Execution::new("sleep 0.1 && echo OK3")
                        .timeout(Some(Duration::from_millis(60))),
                    Execution::new("sleep 0.1 && echo OK4")
                        .timeout(Some(Duration::from_millis(200))),
                ],
                None,
                Ok(vec![
                    Duration::from_millis(50).into(),
                    ("OK2\n", "").into(),
                    Duration::from_millis(60).into(),
                    ("OK4\n", "").into(),
                ]),
            ),
        ];

        run_executor_tests(
            ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: 1,
            },
            tests,
            &ExecutionContext::new(),
        );
    }

    #[test]
    fn test_executes_in_parallel() {
        // Test isn't run in parralel, so it should just take execution * sleep time + general machine load
        let mut tests = vec![(1, 600, 1200)];
        let max = default_parallel_count();
        if max >= 3 {
            tests.push((3, 200, 350));
            if max >= 6 {
                tests.push((6, 100, 250));
            }
        }
        let executions = (1..7)
            .map(|num| Execution::new(&format!("sleep 0.1 && echo OK{}", num)))
            .collect::<Vec<_>>();
        assert_eq!(6, executions.len());

        let lowest = |values: &mut [u128]| {
            values.sort_unstable();
            values[0]
        };

        for (concurrency, min, max) in tests {
            let executor = ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: concurrency,
            };

            // run multiple times, get minimum to account for fuzzy
            let mut values = vec![];
            for i in 1..6 {
                let t = Instant::now();
                let fails = executor
                    .execute_all(
                        &executions.iter().collect::<Vec<_>>(),
                        &ExecutionContext::new(),
                    )
                    .expect("must not error")
                    .iter()
                    .filter(|e| e.exit_code != ExitStatus::SUCCESS)
                    .count();
                let value = t.elapsed().as_millis();
                values.push(value);
                assert_eq!(
                    0, fails,
                    "all executions succeeded for concurrency = {} at run {}",
                    concurrency, i,
                );
            }
            let diff = lowest(&mut values);
            assert!(
                diff >= min,
                "min {}ms with concurrency = {}, got {}ms",
                min,
                concurrency,
                diff
            );

            // I don't know what windows is doing, but nothing that can be
            // tested consistently
            #[cfg(not(target_os = "windows"))]
            assert!(
                diff < max,
                "max {}ms with concurrency = {}, got {}ms",
                max,
                concurrency,
                diff
            );
        }
    }

    #[test]
    fn test_skipped_test_returns_skipped_error() {
        let tests = vec![(
            "Sufficient timeout has no effect",
            vec![
                Execution::new("echo OK1"),
                Execution::new("exit 80"),
                Execution::new("echo OK2"),
            ],
            None,
            Err(ExecutionError::Skipped),
        )];

        run_executor_tests(
            ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: 1,
            },
            tests,
            &ExecutionContext::new(),
        );
    }

    #[test]
    fn test_executor_has_no_state() {
        let tests = vec![
            (
                "Environment variable does not persist",
                vec![
                    Execution::new("export FOO=bar"),
                    Execution::new("echo FOO=${FOO:-undefined}"),
                ],
                None,
                Ok(vec![("", "").into(), ("FOO=undefined\n", "").into()]),
            ),
            (
                "Alias does not persist",
                vec![
                    Execution::new("alias foo='echo BAR'"),
                    Execution::new("foo"),
                ],
                None,
                Ok(vec![
                    ("", "").into(),
                    (
                        "",
                        format!("{}: line 1: foo: command not found\n", DEFAULT_SHELL),
                        Some(127),
                    )
                        .into(),
                ]),
            ),
        ];

        run_executor_tests(
            ParallelShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
                count: 1,
            },
            tests,
            &ExecutionContext::new(),
        );
    }
}
