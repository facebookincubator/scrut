use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use tracing::debug;

use super::context::Context as ExecutionContext;
use super::error::ExecutionError;
use super::error::ExecutionTimeout;
use super::executor::Executor;
use super::executor::Result;
use super::executor::DEFAULT_TOTAL_TIMEOUT;
use super::runner::Runner;
use super::subprocess_runner::SubprocessRunner;
use super::DEFAULT_SHELL;
use crate::config::OutputStreamControl;
use crate::config::TestCaseConfig;
use crate::config::DEFAULT_SKIP_DOCUMENT_CODE;
use crate::lossy_string;
use crate::newline::BytesNewline;
use crate::newline::SplitLinesByNewline;
use crate::output::ExitStatus;
use crate::output::Output;
use crate::testcase::TestCase;

// Amount of random characters that will be appended to divider string
const SUFFIX_RANDOM_SIZE: usize = 20;

// Beginning of the divider string, that separates multiple outputs so that
// they can be split and assigned
const DIVIDER_PREFIX: &str = "~~~~~~~~EXECDIVIDER::";
// TODO: make this a static thingy thing
const DIVIDER_PREFIX_BYTES: &[u8] = b"~~~~~~~~EXECDIVIDER::";

/// An executor that runs all shell expressions of the provided executions
/// within a single bash script (within the same bash process).
///
/// The output is then separated by dividing strings, that are printed in
/// between the sequential executions.
///
/// The executor always processes STDOUT and STDERR combined.
///
/// As a result only timeout over all executions is supported, not per
/// execution. Also, if any of the executions calls explicitly to `exit` (or
/// otherwise ends the execution pre-maturely) then the whole script execution
/// is ended and no results for individual executions are assigned.
///
/// !! Caution: Executions that detach (e.g. `nohup expression &`) are likely
/// to mess with the output assignment !!
pub struct BashScriptExecutor(PathBuf);

impl BashScriptExecutor {
    pub fn new(bash_path: &Path) -> Self {
        Self(bash_path.to_owned())
    }
}

impl Default for BashScriptExecutor {
    fn default() -> Self {
        Self::new(*DEFAULT_SHELL)
    }
}

impl Executor for BashScriptExecutor {
    fn execute_all(
        &self,
        testcases: &[&TestCase],
        context: &ExecutionContext,
    ) -> Result<Vec<Output>> {
        let testcase = compile_testcase(testcases, context)?;
        let runner = SubprocessRunner(self.0.to_owned());
        let output = runner
            .run("script", &testcase, context)
            .map_err(|err| ExecutionError::from_execute(err, None, None))?;
        let skip_document_code = testcase
            .config
            .skip_document_code
            .unwrap_or(DEFAULT_SKIP_DOCUMENT_CODE);
        match output.exit_code {
            ExitStatus::Code(code) if code == skip_document_code => {
                return Err(ExecutionError::Skipped(0));
            }
            ExitStatus::Timeout(_) => return Err(ExecutionError::Timeout(ExecutionTimeout::Total)),
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
        for (index, output) in outputs.iter().enumerate() {
            if output.exit_code == ExitStatus::Code(skip_document_code) {
                return Err(ExecutionError::Skipped(index));
            }
        }

        // check for malformed
        if outputs.len() != testcases.len() {
            // debug!("---- SCRIPT\n{}\n", &expression);
            return Err(ExecutionError::aborted(
                anyhow!(
                    "expected {} execution result(s) but found {}",
                    testcases.len(),
                    outputs.len()
                ),
                Some(output),
            ));
        }

        if testcase.config.output_stream != Some(OutputStreamControl::Combined) {
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

/// Reduce a list of [`TestCase`] into a single one that has as it's shell
/// expression a compiled bash script that executes all expressions and that
/// uses a shared configuration
fn compile_testcase(testcases: &[&TestCase], context: &ExecutionContext) -> Result<TestCase> {
    let mut config = TestCaseConfig::empty();

    // iterate all test cases and make sure that they have a consistent configuration
    // as there is no support for a divergent, per-testcase config.
    for (index, testcase) in testcases.iter().enumerate() {
        macro_rules! set_consistent {
            ($attrib:ident) => {
                if config.$attrib.is_none() {
                    config.$attrib = testcase.config.$attrib.clone();
                } else if config.$attrib != testcase.config.$attrib {
                    return Err(ExecutionError::failed(
                        index,
                        anyhow!(
                            "inconsistent configuration value for {}",
                            stringify!($attrib),
                        ),
                    ));
                }
            };
        }
        set_consistent!(detached);
        set_consistent!(keep_crlf);
        set_consistent!(output_stream);
        set_consistent!(skip_document_code);
        set_consistent!(wait);
        if !config.environment.is_empty() && config.environment != testcase.config.environment {
            return Err(ExecutionError::failed(
                index,
                anyhow!("inconsistent value for environment"),
            ));
        }
        config.environment = testcase.config.environment.clone();
    }

    let timeout = context
        .config
        .total_timeout
        .unwrap_or(*DEFAULT_TOTAL_TIMEOUT);
    if !timeout.is_zero() {
        config.timeout = Some(timeout);
    }

    let script = compile_script(testcases, &config)?;

    Ok(TestCase {
        title: "Test Script".into(),
        shell_expression: script,
        config,
        ..Default::default()
    })
}

/// Compiles all shell expressions of a list of [`TestCase`]s into a single bash script
fn compile_script(testcases: &[&TestCase], config: &TestCaseConfig) -> Result<String> {
    use std::borrow::Cow;

    let mut expressions = vec![];
    let salt = random_string(SUFFIX_RANDOM_SIZE);
    for (index, testcase) in testcases.iter().enumerate() {
        /* if let Some(ref stream) = testcase.config.output_stream {
            if stream != &OutputStreamControl::Combined {
                return Err(ExecutionError::failed(
                    index,
                    anyhow!("bash-script execution supports only the combined output streams",),
                ));
            }
        } */

        if testcase.config.timeout.is_some() {
            return Err(ExecutionError::failed(
                index,
                anyhow!("timeout per execution not supported in bash-script execution",),
            ));
        }

        // add exported environment variables before expression
        let mut unset = vec![];
        for (key, value) in &testcase.config.environment {
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

        // add actual expression
        expressions.push(testcase.shell_expression.to_string());

        // add footer that divides from next execution and captures exit code
        let footer = generate_divider(&salt, index);
        expressions.push("".to_string());
        expressions.push(format!(r#"echo "{}""#, &footer));
        if config.output_stream != Some(OutputStreamControl::Combined) {
            expressions.push(format!(r#"1>&2 echo "{}""#, &footer));
        }
        expressions.append(&mut unset);
    }

    Ok(expressions.join("\n"))
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
    use regex::Regex;

    use super::parse_divider_bytes;
    use super::BashScriptExecutor;
    use super::DividerSearch;
    use super::DIVIDER_PREFIX;
    use crate::config::TestCaseConfig;
    use crate::executors::error::ExecutionError;
    use crate::executors::error::ExecutionTimeout;
    use crate::executors::executor::tests::combined_output_test_suite;
    use crate::executors::executor::tests::run_executor_tests;
    use crate::executors::executor::tests::standard_output_test_suite;
    use crate::formatln;
    use crate::output::ExitStatus;
    use crate::testcase::TestCase;

    #[test]
    fn test_standard_output_test_suite() {
        standard_output_test_suite(BashScriptExecutor::default());
    }

    #[test]
    fn test_combined_output_test_suite() {
        combined_output_test_suite(BashScriptExecutor::default());
    }

    #[test]
    fn test_executor_respects_timeout() {
        let tests = vec![
            (
                "Total timeout is respected",
                vec![
                    TestCase::from_expression("sleep 1.0 && echo OK1"),
                    TestCase::from_expression("sleep 1.0 && echo OK2"),
                    TestCase::from_expression("sleep 1.0 && echo OK3"),
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
                Some(Duration::from_millis(1000)),
                Ok(vec![
                    ("OK1\n", "").into(),
                    ("OK2\n", "").into(),
                    ("OK3\n", "").into(),
                ]),
            ),
        ];

        run_executor_tests(BashScriptExecutor::default(), tests);
    }

    #[test]
    fn test_does_not_support_timeout_per_execution() {
        let tests = vec![(
            "Sufficient timeout has no effect",
            vec![TestCase {
                title: "Test".into(),
                shell_expression: "sleep 0.1 && echo OK1".into(),
                config: TestCaseConfig {
                    timeout: Some(Duration::from_millis(200)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            None,
            Err(ExecutionError::failed(
                0,
                anyhow!("timeout per execution not supported in bash-script execution"),
            )),
        )];

        run_executor_tests(BashScriptExecutor::default(), tests);
    }

    #[test]
    fn test_skipped_test_returns_skipped_error() {
        let tests = vec![(
            "Sufficient timeout has no effect",
            vec![
                TestCase::from_expression("echo OK1"),
                TestCase::from_expression("exit 80"),
                TestCase::from_expression("echo OK2"),
            ],
            None,
            // sequential cannot identify which of the tests returned an error
            Err(ExecutionError::Skipped(0)),
        )];

        run_executor_tests(BashScriptExecutor::default(), tests);
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
                    TestCase::from_expression("shopt -s expand_aliases"),
                    TestCase::from_expression("alias foo='echo BAR'"),
                    TestCase::from_expression("alias"),
                    TestCase::from_expression("foo"),
                    TestCase::from_expression("unalias foo"),
                    TestCase::from_expression("foo"),
                ],
                None,
                Ok(vec![
                    ("", "").into(),
                    ("", "").into(),
                    ("alias foo='echo BAR'\n", "").into(),
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

        run_executor_tests(BashScriptExecutor::default(), tests);
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

    #[test]
    fn test_non_printable_ascii_in_output() {
        let tests = vec![(
            "Skip ends execution",
            vec![
                TestCase::from_expression("echo \"😊🦀\""),
                TestCase::from_expression("echo -e \"A\r\nB\""),
            ],
            None,
            Ok(vec![("😊🦀\n", "").into(), ("A\nB\n", "").into()]),
        )];

        run_executor_tests(BashScriptExecutor::default(), tests);
    }
}
