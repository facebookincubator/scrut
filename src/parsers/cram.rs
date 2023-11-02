use std::sync::Arc;

use anyhow::Result;
use tracing::debug;

use super::line_parser::is_comment;
use super::line_parser::LineParser;
use super::parser::Parser;
use crate::config::DocumentConfig;
use crate::config::TestCaseConfig;
use crate::expectation::ExpectationMaker;
use crate::testcase::TestCase;

pub const DEFAULT_CRAM_INDENTION: usize = 2;

/// A parser for Cram `.t` files, which reads [`crate::testcase::TestCase`]s
/// that are encoded in the form:
///
/// ```cram
/// A title
///   $ command
///   expectation
/// ```
pub struct CramParser {
    expectation_maker: Arc<ExpectationMaker>,
    indention: usize,
}

impl CramParser {
    pub fn new(expectation_maker: Arc<ExpectationMaker>, indention: usize) -> Self {
        CramParser {
            expectation_maker,
            indention,
        }
    }

    pub fn default_new(expectation_maker: Arc<ExpectationMaker>) -> Self {
        Self::new(expectation_maker, DEFAULT_CRAM_INDENTION)
    }
}

/* impl Default for CramParser {
    fn default() -> Self {
        Self::new(2)
    }
} */

impl Parser for CramParser {
    /// See [`super::parser::Parser::parse`]
    fn parse(&self, text: &str) -> Result<(DocumentConfig, Vec<TestCase>)> {
        let mut engine = LineParser::new(self.expectation_maker.clone(), true);
        let lines = text.lines().collect::<Vec<_>>();
        let indent = " ".repeat(self.indention);
        debug!("parsing {} lines of cram file", lines.len());

        for (index, line) in lines.iter().enumerate() {
            if is_comment(line) {
                continue;
            }

            // empty line (or comment) can signify end of testcase
            if line.is_empty() {
                if engine.has_testcase_body() {
                    engine.end_testcase(index)?;
                }
                continue;
            }

            // starting with indentions, means either collecting testcase
            // shell expression or testcase output expectations:
            if let Some(line) = line.strip_prefix(&indent) {
                engine.set_testcase_config(TestCaseConfig::default_cram());
                engine.add_testcase_body(line, index)?;
                continue;
            }

            // not indented, not empty line means: next testcase starts
            engine.end_testcase(index)?;

            // title for the NEXT testcase
            engine.set_testcase_title(line);
        }

        if engine.has_testcase_body() {
            engine.set_testcase_config(TestCaseConfig::default_cram());
            engine.end_testcase(lines.len())?
        }
        debug!("found {} testcases in cram file", engine.testcases.len());

        Ok((DocumentConfig::default(), engine.testcases.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::CramParser;
    use crate::config::TestCaseConfig;
    use crate::expectation::tests::expectation_maker;
    use crate::parsers::cram::DEFAULT_CRAM_INDENTION;
    use crate::parsers::parser::Parser;
    use crate::test_expectation;
    use crate::testcase::TestCase;

    fn parser() -> CramParser {
        let maker = expectation_maker();
        CramParser::new(Arc::new(maker), DEFAULT_CRAM_INDENTION)
    }

    #[test]
    fn test_minimal_testcase() {
        let cram_test = r#"This is a title
  $ echo hello
  hello
"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 2,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
    }

    #[test]
    fn test_allow_empty_lines_between_title_and_body() {
        let cram_test = r#"


This is a title


  $ echo hello
  hello


"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 7,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
    }

    #[test]
    fn test_the_closest_title_is_used() {
        let cram_test = r#"

Title 1

Title 2


  $ echo hello
  hello


"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "Title 2".to_string(),
                exit_code: None,
                line_number: 8,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
    }

    #[test]
    fn test_multiple_testcases() {
        let cram_test = r#"
This is a title
  $ echo hello
  hello



This is the next title
  $ echo something
  something
This is the yet more title
  $ echo lastly
  lastly
"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(3, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 3,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0],
            "testcase 1",
        );
        assert_eq!(
            TestCase {
                shell_expression: "echo something".to_string(),
                expectations: vec![test_expectation!("equal", "something", false, false)],
                title: "This is the next title".to_string(),
                exit_code: None,
                line_number: 9,
                config: TestCaseConfig::default_cram(),
            },
            testcases[1],
            "testcase 2",
        );
        assert_eq!(
            TestCase {
                shell_expression: "echo lastly".to_string(),
                expectations: vec![test_expectation!("equal", "lastly", false, false)],
                title: "This is the yet more title".to_string(),
                exit_code: None,
                line_number: 12,
                config: TestCaseConfig::default_cram(),
            },
            testcases[2],
            "testcase 3",
        );
    }

    #[test]
    fn test_multiline_command() {
        let cram_test = r"
The title
  $ echo hello && \
  > echo more && \
  > echo most
  hello
  more
  most
";
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: ["echo hello && \\", "echo more && \\", "echo most"].join("\n"),
                expectations: vec![
                    test_expectation!("equal", "hello", false, false),
                    test_expectation!("equal", "more", false, false),
                    test_expectation!("equal", "most", false, false),
                ],
                title: "The title".into(),
                exit_code: None,
                line_number: 3,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
    }

    #[test]
    fn test_exit_code_is_extracted() {
        let cram_test = r#"
This has an exit code 1
  $ command1
  output
  [4]

This has an exit code 2
  $ command2
  [15]

This has an exit code 3
  $ command3
  output1
  [106]
  output2
"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(3, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "command1".to_string(),
                expectations: vec![test_expectation!("equal", "output", false, false)],
                title: "This has an exit code 1".to_string(),
                exit_code: Some(4),
                line_number: 3,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
        assert_eq!(
            TestCase {
                shell_expression: "command2".to_string(),
                expectations: vec![],
                title: "This has an exit code 2".to_string(),
                exit_code: Some(15),
                line_number: 8,
                config: TestCaseConfig::default_cram(),
            },
            testcases[1]
        );
        assert_eq!(
            TestCase {
                shell_expression: "command3".to_string(),
                expectations: vec![
                    test_expectation!("equal", "output1", false, false),
                    test_expectation!("equal", "output2", false, false)
                ],
                title: "This has an exit code 3".to_string(),
                exit_code: Some(106),
                line_number: 12,
                config: TestCaseConfig::default_cram(),
            },
            testcases[2]
        );
    }

    #[test]
    fn test_only_one_exit_code_is_allowed() {
        let cram_test = r#"
Only one exit code please
  $ command1
  [1]
  [2]
"#;
        let parser = parser();
        let result = parser.parse(cram_test);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            "line 5: exit code provided multiple times".to_string(),
            err.to_string()
        );
    }

    #[test]
    fn test_line_comments_are_ignored() {
        let cram_test = r#"
# this is a line comment
# and another line comment

This is a title
  $ echo hello
  # this is not a comment, but part of the output
  hello

# even more line comments
"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![
                    test_expectation!(
                        "equal",
                        "# this is not a comment, but part of the output",
                        false,
                        false
                    ),
                    test_expectation!("equal", "hello", false, false),
                ],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 6,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
    }

    #[test]
    fn test_real_life_multiline() {
        let cram_test = r#"Setup a buck dir with a mock visibility list
  $ source $TESTDIR/setup.sh
  $ mkdir -p path/to
  $ cat << EOF > path/to/visibility.bzl
  > VISIBILITY = [
  >     "//:bar",
  >     "//:baz",
  > ]
  > EOF
  $ bla"#;
        let parser = parser();
        let (_, testcases) = parser.parse(cram_test).expect("no error");

        assert_eq!(4, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "source $TESTDIR/setup.sh".to_string(),
                expectations: vec![],
                title: "Setup a buck dir with a mock visibility list".to_string(),
                exit_code: None,
                line_number: 2,
                config: TestCaseConfig::default_cram(),
            },
            testcases[0]
        );
        assert_eq!(
            TestCase {
                shell_expression: "mkdir -p path/to".to_string(),
                expectations: vec![],
                title: "".to_string(),
                exit_code: None,
                line_number: 3,
                config: TestCaseConfig::default_cram(),
            },
            testcases[1]
        );
        assert_eq!(
            TestCase {
                shell_expression: [
                    "cat << EOF > path/to/visibility.bzl",
                    "VISIBILITY = [",
                    "    \"//:bar\",",
                    "    \"//:baz\",",
                    "]",
                    "EOF",
                ]
                .join("\n"),
                expectations: vec![],
                title: "".to_string(),
                exit_code: None,
                line_number: 4,
                config: TestCaseConfig::default_cram(),
            },
            testcases[2]
        );
        assert_eq!(
            TestCase {
                shell_expression: ["bla"].join("\n"),
                expectations: vec![],
                title: "".to_string(),
                exit_code: None,
                line_number: 10,
                config: TestCaseConfig::default_cram(),
            },
            testcases[3]
        );
    }
}
