use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use tracing::debug;

use super::context::Context as ExecutionContext;
use super::error::ExecutionError;
use super::execution::Execution;
use super::executor::Executor;
use super::executor::Result;
use super::shell::run_in_shell;
use super::DEFAULT_SHELL;
use crate::lossy_string;
use crate::newline::BytesNewline;
use crate::newline::SplitLinesByNewline;
use crate::output::ExitStatus;
use crate::output::Output;

// Amount of random characters that will be appended to divider string
const SUFFIX_RANDOM_SIZE: usize = 20;

// Beginning of the divider string, that separates multiple outputs so that
// they can be split and assigned
const DIVIDER_PREFIX: &str = "~~~~~~~~EXECDIVIDER::";
// TODO: make this a static thingy thing
const DIVIDER_PREFIX_BYTES: &[u8] = b"~~~~~~~~EXECDIVIDER::";
/* lazy_static! {
    static ref DIVIDER_PREFIX_BYTES: &'static [u8] = DIVIDER_PREFIX.as_bytes();
} */

/// Executions in a sequential shell share environment variables and aliases.
/// All executions happen in sequential order and cannot have individual timeouts.
pub struct SequentialShellExecutor {
    /// Path to shell to use
    pub shell: String,
}

impl SequentialShellExecutor {
    pub fn new(shell: &str) -> Self {
        Self {
            shell: shell.to_string(),
        }
    }
}

impl Default for SequentialShellExecutor {
    fn default() -> Self {
        Self::new(DEFAULT_SHELL)
    }
}

impl Executor for SequentialShellExecutor {
    /// Run all Executions in given order. Timeout over all Executions is supported.
    /// Timeout per Execution is not.
    fn execute_all(
        &self,
        executions: &[&Execution],
        context: &ExecutionContext,
    ) -> Result<Vec<Output>> {
        let expression = self.build_expression(executions, context)?;

        let output = run_in_shell(
            &self.shell,
            &Execution::new(&expression).timeout(context.timeout),
            context,
        )
        .map_err(|err| ExecutionError::from_execute(err, None, None))?;

        match output.exit_code {
            ExitStatus::SKIP => return Err(ExecutionError::Skipped),
            ExitStatus::Timeout(_) => return Err(ExecutionError::Timeout),
            ExitStatus::Unknown => {
                return Err(ExecutionError::aborted(
                    anyhow!("execution failed"),
                    Some(output),
                ));
            }
            _ => {}
        }

        // iterate STDOUT and split by divider string
        let mut outputs = vec![];
        iterate_divided_output(
            "STDOUT",
            (&output.stdout).into(),
            |_index: usize, out: &[u8], exit_code: i32| {
                outputs.push(Output {
                    stderr: vec![].into(),
                    stdout: out.to_vec().into(),
                    exit_code: ExitStatus::Code(exit_code),
                });
                Ok(())
            },
        )?;

        // skip this?
        if outputs
            .iter()
            .any(|output| output.exit_code == ExitStatus::SKIP)
        {
            return Err(ExecutionError::Skipped);
        }

        // check for malformed
        if outputs.len() != executions.len() {
            // debug!("---- SCRIPT\n{}\n", &expression);
            return Err(ExecutionError::aborted(
                anyhow!(
                    "expected {} execution result(s) but found {}",
                    executions.len(),
                    outputs.len()
                ),
                Some(output),
            ));
        }

        if !context.combine_output {
            iterate_divided_output(
                "STDERR",
                (&output.stderr).into(),
                |index: usize, out: &[u8], _exit_code: i32| {
                    if index >= outputs.len() {
                        return Err(ExecutionError::aborted(
                            anyhow!(
                                "expected {} STDERR outputs, but got at least {}",
                                outputs.len(),
                                index + 1
                            ),
                            Some(outputs[outputs.len() - 1].clone()),
                        ));
                    }
                    outputs[index].stderr = out.to_vec().into();
                    Ok(())
                },
            )?;
        }

        Ok(outputs)
    }
}

impl SequentialShellExecutor {
    /// Create a shell script, that contains all executions and appends printing
    /// of the divider after each individual one, so that the execution of the
    /// generated script prints out the results of the individual executions,
    /// divided by a known string
    ///
    /// This method creates expressions that work within bash-like shells. That
    /// means the "normal" `export VARNAME=VARVALUE` syntax is being used and
    /// needs to be supported.
    fn build_expression(
        &self,
        executions: &[&Execution],
        context: &ExecutionContext,
    ) -> Result<String> {
        use std::borrow::Cow;

        let mut expressions = vec![];
        let salt = random_string(SUFFIX_RANDOM_SIZE);
        for (index, execution) in executions.iter().enumerate() {
            if execution.timeout.is_some() {
                return Err(ExecutionError::failed(
                    index,
                    anyhow!("timeout per execution not supported in sequential execution",),
                ));
            }

            // add exported environment variables before expression
            let mut unset = vec![];
            if let Some(ref environment) = execution.environment {
                for (key, value) in environment {
                    // variable keys and values are assumed to be escaped in bash-like
                    // environments, that means even when executing in windows within
                    // a `bash.exe` process, the unix escaping is needed
                    let qkey = shell_escape::unix::escape(Cow::from(key)).to_string();
                    if qkey != *key {
                        return Err(ExecutionError::failed(
                            index,
                            anyhow!("Environment variable {} contains invalid characters", &qkey),
                        ));
                    }
                    let qval = shell_escape::unix::escape(Cow::from(value)).to_string();
                    expressions.push(format!("export {}={}", &qkey, &qval));
                    unset.push(format!("unset {}", &qkey));
                }
            }

            // add actual expression
            expressions.push(execution.expression.to_string());

            // add footer that divides from next execution and captures exit code
            let footer = generate_divider(&salt, index);
            expressions.push("".to_string());
            expressions.push(format!(r#"echo "{}""#, &footer));
            if !context.combine_output {
                expressions.push(format!(r#"1>&2 echo "{}""#, &footer));
            }
            expressions.append(&mut unset);
        }

        Ok(expressions.join("\n"))
    }
}

fn iterate_divided_output<C>(name: &str, output: &[u8], mut callback: C) -> Result<()>
where
    C: FnMut(usize, &[u8], i32) -> Result<()>,
{
    let mut buffer = vec![];
    let mut expected_index = 0;
    for line in output.split_at_newline() {
        let divider =
            parse_divider_bytes(line).map_err(|err| ExecutionError::failed(expected_index, err))?;
        match divider {
            DividerSearch::NotFound => buffer.push(line.to_vec()),
            DividerSearch::Found {
                prefix,
                output_index,
                exit_code,
            } => {
                if output_index != expected_index {
                    debug!("---- {}\n{}\n----", name, lossy_string!(output));
                    return Err(ExecutionError::failed(
                        output_index,
                        anyhow!(
                            "unexpected result in {} (expected index {}, found {})",
                            name,
                            expected_index,
                            output_index
                        ),
                    ));
                }
                let mut output = if !buffer.is_empty() {
                    buffer.iter().flatten().copied().collect()
                } else {
                    vec![]
                };
                if let Some(mut prefix) = prefix {
                    output.append(&mut prefix);
                }
                callback(output_index, &output, exit_code)?;
                expected_index += 1;
                buffer.clear();
            }
        }
    }
    Ok(())
}

/// Create a new divider that separated outputs of multiple executions
fn generate_divider(salt: &str, index: usize) -> String {
    format!("{}{}::{}::$?", DIVIDER_PREFIX, salt, index)
}

#[derive(Debug, PartialEq)]
enum DividerSearch {
    Found {
        prefix: Option<Vec<u8>>,
        output_index: usize,
        exit_code: i32,
    },
    NotFound,
}

/// Extracts index and exit code from lines that contain the divider. Output
/// lines that do not end in a new line may have the divider appended, in
/// which case the line prefix is return as a non-empty String
fn parse_divider_bytes(line: &[u8]) -> anyhow::Result<DividerSearch> {
    let line = line.trim_newlines();
    let index = line
        .windows(DIVIDER_PREFIX_BYTES.len())
        .position(|window| window == DIVIDER_PREFIX_BYTES);
    if index.is_none() {
        return Ok(DividerSearch::NotFound);
    }

    // extract prefix
    let index = index.unwrap();
    let prefix = if index > 0 {
        Some(line[0..index].to_vec())
    } else {
        None
    };

    // skip after prefix
    let line = &line[index + DIVIDER_PREFIX_BYTES.len()..];

    // skip salt
    let index = line.windows(2).position(|window| window == b"::");
    if index.is_none() {
        bail!("salt is missing from divider line");
    }
    let index = index.unwrap();
    let line = &line[index + 2..];

    // get index and exit code
    let index = line.windows(2).position(|window| window == b"::");
    if index.is_none() {
        bail!("output index is missing from divider line");
    }
    let index = index.unwrap();
    let output_index =
        String::from_utf8(line[0..index].to_vec()).context("output index must be utf8")?;
    let exit_code =
        String::from_utf8(line[index + 2..].to_vec()).context("return code index must be utf8")?;

    Ok(DividerSearch::Found {
        prefix,
        output_index: output_index
            .parse::<usize>()
            .with_context(|| format!("parse divider output index {}", output_index))?,
        exit_code: exit_code
            .parse::<i32>()
            .with_context(|| format!("parse divider exit code {}", exit_code))?,
    })
}

/// Generate a random alphanumeric string of given size
fn random_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::anyhow;

    use super::parse_divider_bytes;
    use super::DividerSearch;
    use super::SequentialShellExecutor;
    use super::DIVIDER_PREFIX;
    use crate::executors::context::Context as ExecutionContext;
    use crate::executors::error::ExecutionError;
    use crate::executors::execution::Execution;
    use crate::executors::executor::tests::combined_output_test_suite;
    use crate::executors::executor::tests::run_executor_tests;
    use crate::executors::executor::tests::standard_test_suite;
    use crate::executors::DEFAULT_SHELL;
    use crate::formatln;

    #[test]
    fn test_standard_test_suite() {
        standard_test_suite(SequentialShellExecutor::default(), &ExecutionContext::new());
    }

    #[test]
    fn test_combined_output_test_suite() {
        combined_output_test_suite(
            SequentialShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
            },
            &ExecutionContext::new().combine_output(true),
        );
    }

    #[test]
    fn test_executor_respects_timeout() {
        let tests = vec![
            (
                "Total timeout is respected",
                vec![
                    Execution::new("sleep 1.0 && echo OK1"),
                    Execution::new("sleep 1.0 && echo OK2"),
                    Execution::new("sleep 1.0 && echo OK3"),
                ],
                Some(Duration::from_millis(150)),
                Err(ExecutionError::Timeout),
            ),
            (
                "Execution within timeout",
                vec![
                    Execution::new("sleep 0.1 && echo OK1"),
                    Execution::new("sleep 0.1 && echo OK2"),
                    Execution::new("sleep 0.1 && echo OK3"),
                ],
                // windows execution takes a long time to start up, test intends
                // to assert that timeout > actual execution does not return
                // a timeout error -> long timeout is fine
                Some(Duration::from_millis(1000)),
                Ok(vec![
                    ("OK1\n", "").into(),
                    ("OK2\n", "").into(),
                    ("OK3\n", "").into(),
                ]),
            ),
        ];

        run_executor_tests(
            SequentialShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
            },
            tests,
            &ExecutionContext::new(),
        );
    }

    #[test]
    fn test_does_not_support_timeout_per_execution() {
        let tests = vec![(
            "Sufficient timeout has no effect",
            vec![Execution::new("sleep 0.1 && echo OK1").timeout(Some(Duration::from_millis(200)))],
            None,
            Err(ExecutionError::failed(
                0,
                anyhow!("timeout per execution not supported in sequential execution"),
            )),
        )];

        run_executor_tests(
            SequentialShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
            },
            tests,
            &ExecutionContext::new(),
        );
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
            SequentialShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
            },
            tests,
            &ExecutionContext::new(),
        );
    }

    #[test]
    fn test_executor_keeps_state() {
        let tests = vec![
            (
                "Environment variable persists",
                vec![
                    Execution::new("export FOO=bar"),
                    Execution::new("echo FOO=${FOO:-undefined}"),
                    Execution::new("unset FOO"),
                    Execution::new("echo FOO=${FOO:-undefined}"),
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
                "Alias persists",
                vec![
                    Execution::new("shopt -s expand_aliases"),
                    Execution::new("alias foo='echo BAR'"),
                    Execution::new("foo"),
                    Execution::new("unalias foo"),
                    Execution::new("foo"),
                ],
                None,
                Ok(vec![
                    ("", "").into(),
                    ("", "").into(),
                    ("BAR\n", "").into(),
                    ("", "").into(),
                    (
                        "",
                        format!("{}: line 17: foo: command not found\n", DEFAULT_SHELL),
                        Some(127),
                    )
                        .into(),
                ]),
            ),
        ];

        run_executor_tests(
            SequentialShellExecutor {
                shell: DEFAULT_SHELL.to_string(),
            },
            tests,
            &ExecutionContext::new(),
        );
    }

    #[test]
    fn test_parse_divider_bytes() {
        let tests = vec![
            ("foo".to_string(), DividerSearch::NotFound),
            (
                format!("{}abcd::5::12", DIVIDER_PREFIX),
                DividerSearch::Found {
                    prefix: None,
                    output_index: 5,
                    exit_code: 12,
                },
            ),
            (
                formatln!("{}abcd::981::128", DIVIDER_PREFIX),
                DividerSearch::Found {
                    prefix: None,
                    output_index: 981,
                    exit_code: 128,
                },
            ),
            (
                formatln!("something{}abcd::123::234", DIVIDER_PREFIX),
                DividerSearch::Found {
                    prefix: Some(b"something".to_vec()),
                    output_index: 123,
                    exit_code: 234,
                },
            ),
        ];
        for (divider, expect) in tests {
            let result = parse_divider_bytes(divider.as_bytes()).expect("parse line");
            assert_eq!(expect, result, "from `{}`", divider)
        }
    }
}
