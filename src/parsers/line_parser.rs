/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::sync::Arc;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use regex::Regex;

use crate::config::TestCaseConfig;
use crate::expectation::Expectation;
use crate::expectation::ExpectationMaker;
use crate::testcase::TestCase;

lazy_static! {
    /// Exit code expression matches an output line of the form:
    ///
    /// ```bnf
    /// <exit-code-expression> ::= "[" <integer> "]"
    /// ```
    static ref EXIT_CODE_EXPRESSION: Regex =
        Regex::new("^\\[([0-9]+)\\]$").expect("exit code expression must compile");
}

pub(super) enum CodeType {
    CommandStart,
    CommandContinue,
    Expectation,
    ExitCode,
}

/// A meta parser engine, that can be used for any line-by-line test file format
/// which uses the standard Cram-like encoding of testcase bodies like:
///
/// ```txt
/// $ a-command \
/// > potential more lines
/// other lines are
/// all output
/// ```
pub(super) struct LineParser {
    pub(super) testcases: Vec<TestCase>,
    expectation_maker: Arc<ExpectationMaker>,
    title: Option<String>,
    command: Vec<String>,
    exit_code: Option<i32>,
    expectations: Vec<Expectation>,
    in_command: bool,
    allow_multiple_commands: bool,
    output_start_index: Option<usize>,
    config: Option<TestCaseConfig>,
}

impl LineParser {
    pub(super) fn new(
        expectation_maker: Arc<ExpectationMaker>,
        allow_multiple_commands: bool,
    ) -> Self {
        Self {
            expectation_maker,
            title: None,
            command: vec![],
            expectations: vec![],
            exit_code: None,
            testcases: vec![],
            in_command: false,
            allow_multiple_commands,
            output_start_index: None,
            config: None,
        }
    }

    /// Add a line that is either a command or an expectation
    pub(super) fn add_testcase_body(&mut self, line: &str, index: usize) -> Result<CodeType> {
        // start of command
        if self.allow_multiple_commands || self.command.is_empty() {
            if let Some(line) = line.strip_prefix("$ ") {
                self.in_command = true;
                if !self.command.is_empty() {
                    self.end_testcase(index)?;
                }
                if self.output_start_index.is_none() {
                    self.output_start_index = Some(index);
                }
                self.command.push(line.into());
                return Ok(CodeType::CommandStart);
            }
        }

        // continuation of command
        if self.in_command {
            if let Some(line) = line.strip_prefix("> ") {
                if self.command.is_empty() {
                    bail!(
                        "line {}: command extender '>' requires previous command start '$' which is not given",
                        index + 1
                    );
                }
                self.command.push(line.into());
                return Ok(CodeType::CommandContinue);
            }
        }

        self.in_command = false;
        if let Some(exit_code) = extract_exit_code(line) {
            if self.exit_code.is_some() {
                bail!("line {}: exit code provided multiple times", index + 1)
            }
            self.exit_code = Some(exit_code);
            return Ok(CodeType::ExitCode);
        }

        self.expectations.push(
            self.expectation_maker
                .parse(line)
                .with_context(|| format!("parsing line {}", index + 1))?,
        );
        Ok(CodeType::Expectation)
    }

    /// Add a line of title
    pub(super) fn set_testcase_title(&mut self, line: &str) {
        self.title = Some(line.to_string())
    }

    /// Add a line of title
    pub(super) fn set_testcase_config(&mut self, config: TestCaseConfig) {
        self.config = Some(config)
    }

    /// Signify end of currently processed testcase, which will test the
    /// validity of the testcase, add it to the stack and flush the state
    /// so that the next testcase(s) can be processed.
    pub(super) fn end_testcase(&mut self, line_index: usize) -> Result<()> {
        let (has_commands, has_expectations) =
            (!self.command.is_empty(), !self.expectations.is_empty());
        if !has_commands {
            if has_expectations {
                bail!(
                    "line {}: testcase output expectation(s) given, but no shell expression specified",
                    line_index + 1
                )
            }
            return Ok(());
        }
        self.testcases.push(TestCase {
            title: self.title.to_owned().unwrap_or_default(),
            shell_expression: self.command.join("\n"),
            exit_code: self.exit_code,
            expectations: self.expectations.clone(),
            line_number: self.output_start_index.unwrap_or(line_index) + 1,
            config: self.config.clone().unwrap_or_default(),
        });
        self.flush();
        Ok(())
    }

    // whether shell expression(s) or expectation(s) are given
    pub(super) fn has_testcase_body(&self) -> bool {
        !self.command.is_empty() || !self.expectations.is_empty()
    }

    fn flush(&mut self) {
        self.title = None;
        self.command = vec![];
        self.expectations = vec![];
        self.exit_code = None;
        self.output_start_index = None;
        self.config = None;
    }
}

/// Parse a line of output for whether it contains an exit code of
/// the form `[<numeric code>]` and return the numeric value if it does
pub(super) fn extract_exit_code(line: &str) -> Option<i32> {
    // map. and then? map! and then?? map!!1!1!!!1 and ... then? ERRRR
    EXIT_CODE_EXPRESSION
        .captures(line)
        .and_then(|captures| {
            captures
                .iter()
                .nth(1)
                .and_then(|matching| matching.map(|matching| matching.as_str()))
        })
        .and_then(|s| s.parse::<i32>().map_or_else(|_| None, Some))
}

/// Lines starting with "#" are considered comments
pub(super) fn is_comment(line: &str) -> bool {
    line.starts_with('#')
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::extract_exit_code;
    use super::LineParser;
    use crate::expectation::tests::expectation_maker;
    use crate::test_expectation;
    use crate::testcase::TestCase;

    fn engine(allow_multiple_commands: bool) -> LineParser {
        let maker = expectation_maker();
        LineParser::new(Arc::new(maker), allow_multiple_commands)
    }

    #[test]
    fn test_testcase_is_combined() {
        let mut engine = engine(false);
        engine.set_testcase_title("foo");
        engine.add_testcase_body("$ bar", 1).expect("add command");
        engine.add_testcase_body("baz", 2).expect("add expectation");
        engine.add_testcase_body("[5]", 3).expect("add expectation");
        engine.end_testcase(4).expect("testcase ending");
        assert_eq!(
            vec![TestCase {
                title: "foo".to_string(),
                exit_code: Some(5),
                expectations: vec![test_expectation!("equal", "baz"),],
                shell_expression: "bar".to_string(),
                line_number: 2,
                ..Default::default()
            },],
            engine.testcases,
        )
    }

    #[test]
    fn test_last_title_is_used() {
        let mut engine = engine(false);
        engine.set_testcase_title("foo1");
        engine.set_testcase_title("foo2");
        engine.set_testcase_title("foo3");
        engine.add_testcase_body("$ bar", 1).expect("add command");
        engine.add_testcase_body("baz", 2).expect("add expectation");
        engine.add_testcase_body("[5]", 3).expect("add expectation");
        engine.end_testcase(4).expect("testcase ending");
        assert_eq!(
            vec![TestCase {
                title: "foo3".to_string(),
                exit_code: Some(5),
                expectations: vec![test_expectation!("equal", "baz"),],
                shell_expression: "bar".to_string(),
                line_number: 2,
                ..Default::default()
            },],
            engine.testcases,
        )
    }

    #[test]
    fn test_command_is_combined() {
        let mut engine = engine(false);
        engine.set_testcase_title("foo");
        engine.add_testcase_body("$ bar1", 1).expect("add command");
        engine.add_testcase_body("> bar2", 1).expect("add command");
        engine.add_testcase_body("> bar3", 1).expect("add command");
        engine.add_testcase_body("baz", 2).expect("add expectation");
        engine.add_testcase_body("[5]", 3).expect("add expectation");
        engine.end_testcase(4).expect("testcase ending");
        assert_eq!(
            vec![TestCase {
                title: "foo".to_string(),
                exit_code: Some(5),
                expectations: vec![test_expectation!("equal", "baz"),],
                shell_expression: "bar1\nbar2\nbar3".to_string(),
                line_number: 2,
                ..Default::default()
            },],
            engine.testcases,
        )
    }

    #[test]
    fn test_expectations_are_stacked() {
        let mut engine = engine(false);
        engine.set_testcase_title("foo");
        engine.add_testcase_body("$ bar", 1).expect("add command");
        engine
            .add_testcase_body("baz1", 2)
            .expect("add expectation");
        engine
            .add_testcase_body("baz2", 3)
            .expect("add expectation");
        engine
            .add_testcase_body("baz3", 4)
            .expect("add expectation");
        engine.add_testcase_body("[5]", 5).expect("add expectation");
        engine.end_testcase(6).expect("testcase ending");
        assert_eq!(
            vec![TestCase {
                title: "foo".to_string(),
                exit_code: Some(5),
                expectations: vec![
                    test_expectation!("equal", "baz1"),
                    test_expectation!("equal", "baz2"),
                    test_expectation!("equal", "baz3"),
                ],
                shell_expression: "bar".to_string(),
                line_number: 2,
                ..Default::default()
            },],
            engine.testcases,
        )
    }

    #[test]
    fn test_multiple_commands_in_block() {
        let mut engine = engine(true);
        engine
            .add_testcase_body("$ foo1", 1)
            .expect("add 1st command");
        engine
            .add_testcase_body("$ foo2", 2)
            .expect("add 2nd command");
        engine
            .add_testcase_body("$ foo3", 3)
            .expect("add 3rd command");
        engine.end_testcase(4).expect("testcase ending");
        assert_eq!(
            vec![
                TestCase {
                    title: "".to_string(),
                    exit_code: None,
                    expectations: vec![],
                    shell_expression: "foo1".to_string(),
                    line_number: 2,
                    ..Default::default()
                },
                TestCase {
                    title: "".to_string(),
                    exit_code: None,
                    expectations: vec![],
                    shell_expression: "foo2".to_string(),
                    line_number: 3,
                    ..Default::default()
                },
                TestCase {
                    title: "".to_string(),
                    exit_code: None,
                    expectations: vec![],
                    shell_expression: "foo3".to_string(),
                    line_number: 4,
                    ..Default::default()
                }
            ],
            engine.testcases,
        )
    }

    #[test]
    fn test_single_command_in_block() {
        let mut engine = engine(false);
        engine
            .add_testcase_body("$ foo1", 1)
            .expect("add 1st command");
        engine
            .add_testcase_body("$ foo2", 2)
            .expect("add 2nd command");
        engine
            .add_testcase_body("$ foo3", 3)
            .expect("add 3rd command");
        engine.end_testcase(4).expect("testcase ending");
        assert_eq!(
            vec![TestCase {
                title: "".to_string(),
                exit_code: None,
                expectations: vec![
                    test_expectation!("equal", "$ foo2"),
                    test_expectation!("equal", "$ foo3"),
                ],
                shell_expression: "foo1".to_string(),
                line_number: 2,
                ..Default::default()
            },],
            engine.testcases,
        )
    }

    #[test]
    fn test_testcases_stack() {
        let mut engine = engine(false);
        engine.set_testcase_title("foo1");
        engine.add_testcase_body("$ bar1", 1).expect("add command1");
        engine
            .add_testcase_body("baz1", 2)
            .expect("add expectation1");
        engine.add_testcase_body("[1]", 3).expect("add exit code1");
        engine.end_testcase(10).expect("testcase ending1");
        engine.set_testcase_title("foo2");
        engine.add_testcase_body("$ bar2", 4).expect("add command2");
        engine
            .add_testcase_body("baz2", 5)
            .expect("add expectation2");
        engine.add_testcase_body("[2]", 6).expect("add exit code2");
        engine.end_testcase(10).expect("testcase ending2");
        engine.set_testcase_title("foo3");
        engine.add_testcase_body("$ bar3", 7).expect("add command3");
        engine
            .add_testcase_body("baz3", 8)
            .expect("add expectation3");
        engine.add_testcase_body("[3]", 9).expect("add exit code3");
        engine.end_testcase(10).expect("testcase ending3");
        assert_eq!(
            vec![
                TestCase {
                    title: "foo1".to_string(),
                    exit_code: Some(1),
                    expectations: vec![test_expectation!("equal", "baz1"),],
                    shell_expression: "bar1".to_string(),
                    line_number: 2,
                    ..Default::default()
                },
                TestCase {
                    title: "foo2".to_string(),
                    exit_code: Some(2),
                    expectations: vec![test_expectation!("equal", "baz2"),],
                    shell_expression: "bar2".to_string(),
                    line_number: 5,
                    ..Default::default()
                },
                TestCase {
                    title: "foo3".to_string(),
                    exit_code: Some(3),
                    expectations: vec![test_expectation!("equal", "baz3"),],
                    shell_expression: "bar3".to_string(),
                    line_number: 8,
                    ..Default::default()
                }
            ],
            engine.testcases,
        )
    }

    #[test]
    fn test_exit_code_provided_is_remembered() {
        for provided in [true, false] {
            let mut engine = engine(false);
            engine.set_testcase_title("foo1");
            engine.add_testcase_body("$ bar", 1).expect("add command");
            if provided {
                engine.add_testcase_body("[0]", 2).expect("add exit code");
            }
            engine.end_testcase(3).expect("testcase ending");
            assert_eq!(
                vec![TestCase {
                    title: "foo1".to_string(),
                    exit_code: if provided { Some(0) } else { None },
                    expectations: vec![],
                    shell_expression: "bar".to_string(),
                    line_number: 2,
                    ..Default::default()
                },],
                engine.testcases,
                "provided exit code {provided}",
            )
        }
    }

    #[test]
    fn test_extract_exit_code() {
        let tests: Vec<(&str, Option<i32>)> = vec![
            ("foo", None),
            ("[]", None),
            ("[0]", Some(0)),
            ("[1]", Some(1)),
            ("[99]", Some(99)),
            ("[a]", None),
        ];
        tests.iter().for_each(|(line, expect)| {
            let result = extract_exit_code(line);
            assert_eq!(*expect, result, "parsed '{}'", line);
        });
    }
}
