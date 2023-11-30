/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::borrow::Cow;
use std::fmt::Display;
#[cfg(test)]
use std::time::Duration;

use serde::ser::SerializeMap;
use serde::Serialize;
use serde::Serializer;
use serde_json::json;
use serde_json::Value;

use crate::config::OutputStreamControl;
use crate::config::TestCaseConfig;
use crate::diff::Diff;
use crate::diff::DiffTool;
use crate::expectation::Expectation;
use crate::newline::replace_crlf;
use crate::output::ExitStatus;
use crate::output::Output;

pub type Result<T> = anyhow::Result<T, TestCaseError>;

/// An aggregate that unifies all ingredients for a test: a title
/// of the expected and intended state of the world; what a specific
/// command line should output and why
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TestCase {
    /// A human readable description that clarifies the intention
    pub title: String,

    /// The valid shell expression that is to be executed
    pub shell_expression: String,

    /// The expectations that describe the output of the execution
    pub expectations: Vec<Expectation>,

    /// The expected exit code of the execution
    #[serde(serialize_with = "serialize_always_as_value")]
    pub exit_code: Option<i32>,

    /// The line number of this test in the original file (starting at 1)
    pub line_number: usize,

    /// Configuration that influences the behavior of this test-case
    #[serde(skip_serializing_if = "TestCaseConfig::is_empty")]
    pub config: TestCaseConfig,
}

impl TestCase {
    /// Validate that the outcome of an execution matches with the assumed
    /// outcome in regards to exit code and (STDOUT) output, or return an
    /// [`TestCaseError`]
    pub fn validate(&self, output: &Output) -> Result<()> {
        if let ExitStatus::Code(exit_code) = output.exit_code {
            let expected = self.exit_code.unwrap_or(0);
            if exit_code != expected {
                return Err(TestCaseError::InvalidExitCode {
                    actual: exit_code,
                    expected,
                });
            }
        }
        let diff_tool = DiffTool::new(self.expectations.clone());
        let stream = if self.config.output_stream == Some(OutputStreamControl::Stderr) {
            &output.stderr
        } else {
            &output.stdout
        };
        let diff = diff_tool
            .diff(stream.into())
            .map_err(TestCaseError::InternalError)?;
        if diff.has_differences() {
            Err(TestCaseError::MalformedOutput(diff))
        } else {
            Ok(())
        }
    }

    /// Update the given output by either replacing CRLF with LF or keeping as-is
    /// depending on the [`TestCaseConfig`] settings
    pub fn render_output<'a>(&self, output: &'a [u8]) -> Cow<'a, [u8]> {
        if self.config.keep_crlf == Some(true) {
            Cow::from(output)
        } else {
            replace_crlf(output)
        }
    }

    #[cfg(test)]
    pub fn from_expression(expression: &str) -> Self {
        Self {
            title: "Test".into(),
            shell_expression: expression.into(),
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn from_expression_timed(expression: &str, timeout: Option<Duration>) -> Self {
        Self {
            title: "Test".into(),
            shell_expression: expression.into(),
            config: TestCaseConfig {
                timeout,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub(crate) fn shell_expression_lines(&self) -> usize {
        self.shell_expression.matches('\n').count() + 1
    }

    pub(crate) fn expectations_lines(&self) -> usize {
        self.expectations.len()
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = json!({
            "title": self.title.clone(),
            "shell_expression": self.shell_expression.clone(),
            "expectations": self.expectations
                .iter()
                .map(|e| Value::String(e.to_expression_string(&Default::default())))
                .collect::<Vec<_>>(),
            "exit_code": self.exit_code.unwrap_or(0),
            "line_number": self.line_number,
            "config": &self.config,
        });
        let out = serde_json::to_string(&map).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", out)
    }
}

fn serialize_always_as_value<S>(x: &Option<i32>, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i32(x.unwrap_or(0))
}

/// An error that occurs when the actual output of an execution does not
/// match with the expectations.
///
/// There are four causes why an error can be raised:
/// 1) MalformedOutput: A line of output does not match the expected content or form
/// 2) UnexpectedOutput: There are more lines of output than there are
///    expectations to validate the output. Hence the additional output is
///    unexpected.
/// 3) InsufficientOutput: There are more expectation than there is output. That
///    means some of the expectations could never be applied and must be
///    considered failed (assuming they are non-optional)
/// 4) InternalError: An error occurred during processing, e.g. invalid UTF8
#[derive(Debug)]
pub enum TestCaseError {
    /// The validation of the expectation for the given line failed (invalid input)
    MalformedOutput(Diff),

    /// An execution ends in an unexpected exit code
    InvalidExitCode { actual: i32, expected: i32 },

    /// Delegated internal errors, e.g. relating to decoding
    InternalError(anyhow::Error),

    /// Whether this test was skipped intentionally
    Skipped,
}

impl PartialEq for TestCaseError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MalformedOutput(l0), Self::MalformedOutput(r0)) => l0 == r0,
            (
                Self::InvalidExitCode {
                    actual: l_actual,
                    expected: l_expected,
                },
                Self::InvalidExitCode {
                    actual: r_actual,
                    expected: r_expected,
                },
            ) => l_actual == r_actual && l_expected == r_expected,
            (Self::InternalError(l0), Self::InternalError(r0)) => l0.to_string() == r0.to_string(),
            (_, _) => false,
        }
    }
}

impl Serialize for TestCaseError {
    fn serialize<S>(&self, serializer: S) -> anyhow::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::MalformedOutput(diff) => {
                let mut variant = serializer.serialize_map(Some(2))?;
                variant.serialize_entry("kind", "malformed_output")?;
                variant.serialize_entry("diff", &diff.lines)?;
                variant.end()
            }
            Self::InvalidExitCode { actual, expected } => {
                let mut variant = serializer.serialize_map(Some(3))?;
                variant.serialize_entry("kind", "invalid_exit_code")?;
                variant.serialize_entry("actual", actual)?;
                variant.serialize_entry("expected", expected)?;
                variant.end()
            }
            Self::InternalError(err) => {
                let mut variant = serializer.serialize_map(Some(2))?;
                variant.serialize_entry("kind", "internal_error")?;
                variant.serialize_entry("error", &format!("{}", err))?;
                variant.end()
            }
            Self::Skipped => {
                let mut variant = serializer.serialize_map(Some(1))?;
                variant.serialize_entry("kind", "skipped")?;
                variant.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TestCase;
    use super::TestCaseError;
    use crate::config::TestCaseConfig;
    use crate::diff::Diff;
    use crate::diff::DiffLine;
    use crate::lossy_string;
    use crate::test_expectation;

    #[test]
    fn test_validate_succeeds_on_valid() {
        let testcase = TestCase {
            title: "an testcase".to_string(),
            shell_expression: "a command".to_string(),
            expectations: vec![test_expectation!("no-eol", "the stdout")],
            exit_code: Some(123),
            line_number: 234,
            ..Default::default()
        };
        testcase
            .validate(&("the stdout", "the stderr", Some(123)).into())
            .expect("no error");
    }

    #[test]
    fn test_validate_fails_on_invalid_exit_code() {
        let testcase = TestCase {
            title: "an testcase".to_string(),
            shell_expression: "a command".to_string(),
            expectations: vec![test_expectation!("no-eol", "the stdout", false, false)],
            exit_code: Some(234),
            line_number: 123,
            ..Default::default()
        };
        let asserted_output = ("the stdout", "the stderr", Some(123)).into();
        let result = testcase.validate(&asserted_output);
        match result {
            Ok(_) => panic!("assertion should have failed"),
            Err(err) => match err {
                TestCaseError::InvalidExitCode { actual, expected } => {
                    assert_eq!(
                        asserted_output.exit_code.as_code(),
                        actual,
                        "asserted output is delegated"
                    );
                    assert_eq!(234, expected, "expected exit code is delegated");
                }
                _ => panic!("unexpected error: {:?}", err),
            },
        }
    }

    #[test]
    fn test_validate_fails_on_malformed_output() {
        let testcase = TestCase {
            title: "an testcase".to_string(),
            shell_expression: "a command".to_string(),
            expectations: vec![test_expectation!(
                "no-eol",
                "something not matching",
                false,
                false
            )],
            exit_code: Some(123),
            line_number: 234,
            ..Default::default()
        };
        let asserted_output = ("the stdout", "the stderr", Some(123)).into();
        let result = testcase.validate(&asserted_output);
        match result {
            Ok(_) => panic!("assertion should have failed"),
            Err(err) => {
                assert_eq!(
                    TestCaseError::MalformedOutput(Diff::new(vec![
                        DiffLine::UnmatchedExpectation {
                            index: 0,
                            expectation: testcase.expectations[0].clone()
                        },
                        DiffLine::UnexpectedLines {
                            lines: vec![(0, b"the stdout".to_vec())]
                        },
                    ])),
                    err,
                    "expected exit code is delegated"
                );
            }
        }
    }

    #[test]
    fn test_render_output() {
        let tests = &[
            (false, "foo", "foo"),
            (true, "foo", "foo"),
            (false, "foo\nbar\nbaz", "foo\nbar\nbaz"),
            (true, "foo\nbar\nbaz", "foo\nbar\nbaz"),
            (false, "foo\r\nbar\r\nbaz", "foo\nbar\nbaz"),
            (true, "foo\r\nbar\r\nbaz", "foo\r\nbar\r\nbaz"),
        ];
        for (crlf_support, from, expect) in tests {
            let tc = TestCase {
                title: "an testcase".to_string(),
                shell_expression: "a command".to_string(),
                expectations: vec![test_expectation!("no-eol", "the stdout")],
                exit_code: Some(123),
                line_number: 234,
                config: TestCaseConfig {
                    keep_crlf: Some(*crlf_support),
                    ..Default::default()
                },
            };
            let output = tc.render_output(from.as_bytes());
            assert_eq!(
                *expect,
                lossy_string!(&output),
                "from {} (crlf = {})",
                *from,
                *crlf_support
            );
        }
    }
}
