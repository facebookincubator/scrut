/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Write;

use anyhow::Context;
use anyhow::Result;

use super::generator::TestCaseGenerator;
use super::generator::UpdateGenerator;
use crate::config::TestCaseConfig;
use crate::formatln;
use crate::generators::outcome::OutcomeTestGenerator;
use crate::newline::StringNewline;
use crate::outcome::Outcome;
use crate::parsers::markdown::DEFAULT_MARKDOWN_LANGUAGES;
use crate::parsers::markdown::MarkdownIterator;
use crate::parsers::markdown::MarkdownToken;
use crate::parsers::markdown::NumberedLines;

/// Update [`crate::testcase::TestCase`]s in an existing Markdown document
pub struct MarkdownUpdateGenerator(Vec<String>);

impl MarkdownUpdateGenerator {
    pub fn new(languages: &[&str]) -> Self {
        Self(languages.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
}

impl Default for MarkdownUpdateGenerator {
    fn default() -> Self {
        Self::new(DEFAULT_MARKDOWN_LANGUAGES)
    }
}

impl UpdateGenerator for MarkdownUpdateGenerator {
    fn generate_update(
        &self,
        original_document: &str,
        outcomes: &[&Outcome],
    ) -> anyhow::Result<String> {
        if outcomes.is_empty() {
            return Ok(original_document.into());
        }

        // initialize markdown iterator
        let lines = original_document.lines();
        let languages: &[&str] = &self.0.iter().map(|s| s as &str).collect::<Vec<_>>();
        let iterator = MarkdownIterator::new(languages, lines);

        // iterate all lines of original document ...
        let mut updated = String::new();
        let mut testcase_index = 0;
        for token in iterator {
            match token {
                MarkdownToken::Line(_, line) => updated.push_str(&line.assure_newline()),
                MarkdownToken::DocumentConfig(config) => {
                    let config = config.join_newline();
                    updated.push_str("---\n");
                    updated.push_str(&config);
                    updated.push_str("\n---\n");
                }
                MarkdownToken::VerbatimCodeBlock {
                    starting_line_number: _,
                    language: _,
                    lines,
                } => {
                    for line in lines {
                        updated.push_str(&line.assure_newline());
                    }
                }
                MarkdownToken::TestCodeBlock {
                    language,
                    config_lines,
                    comment_lines,
                    code_lines: _,
                } => {
                    let config = if config_lines.is_empty() {
                        "".into()
                    } else {
                        format!(" {{{}}}", config_lines.join_newline().trim_start())
                    };
                    let generated = outcomes[testcase_index]
                        .generate_testcase()
                        .with_context(|| format!("testcase number {}", testcase_index + 1))?;
                    let backticks = "`".repeat(max_backtick_size(&generated) + 1);
                    updated.push_str(&formatln!("{}{}{}", &backticks, &language, &config));
                    for (_, line) in &comment_lines {
                        updated.push_str(&line.assure_newline());
                    }
                    updated.push_str(&generated);
                    updated.push_str(&backticks.assure_newline());
                    testcase_index += 1;
                }
            }
        }
        Ok(updated)
    }
}

/// Generate a new Markdown [`crate::testcase::TestCase`] document from shell
/// expression and it's [`crate::output::Output`]
pub struct MarkdownTestCaseGenerator(String);

impl MarkdownTestCaseGenerator {
    pub fn new(language: &str) -> Self {
        Self(language.to_string())
    }
}

impl Default for MarkdownTestCaseGenerator {
    fn default() -> Self {
        Self::new(DEFAULT_MARKDOWN_LANGUAGES[0])
    }
}

impl TestCaseGenerator for MarkdownTestCaseGenerator {
    fn generate_testcases(&self, outcomes: &[&Outcome]) -> anyhow::Result<String> {
        let default_config = TestCaseConfig::default_markdown();
        outcomes
            .iter()
            .map(|outcome| {
                let mut rendered = String::new();
                // prefix with title
                if !outcome.testcase.title.is_empty() {
                    write!(rendered, "# {}\n\n", outcome.testcase.title).with_context(|| {
                        format!(
                            "failed to append testcase `{}` to string",
                            outcome.testcase.title
                        )
                    })?;
                }

                let config_diff = outcome.testcase.config.diff(&default_config);
                let config = if config_diff.is_empty() {
                    "".into()
                } else {
                    format!(" {}", config_diff.to_yaml_one_liner())
                };

                // start with shell expressions
                let generated = outcome.generate_testcase()?;
                let backticks = "`".repeat(max_backtick_size(&generated) + 1);
                rendered.push_str(&formatln!("{}{}{}", &backticks, self.0, config));
                rendered.push_str(&generated);
                rendered.push_str(&formatln!("{}", &backticks));
                Ok(rendered)
            })
            .collect::<Result<Vec<_>>>()
            .map(|result| result.join("\n\n"))
    }
}

/// returns the largest amount of backticks in a line that is found in the given
/// code block. If no backtick prefix is found than 2 is return, so that an
/// addition of one to the result always yields the minimal, correct amount of
/// backticks tha are needed to guard the inner code
fn max_backtick_size(code_block: &str) -> usize {
    let mut max = 2;
    for line in code_block.lines() {
        let mut count = 0;
        for ch in line.chars() {
            if ch != '`' {
                break;
            }
            count += 1;
        }
        max = count.max(max)
    }
    max
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use std::time::Duration;

    use super::MarkdownTestCaseGenerator;
    use super::MarkdownUpdateGenerator;
    use crate::config::TestCaseConfig;
    use crate::config::TestCaseWait;
    use crate::diff::Diff;
    use crate::diff::DiffLine;
    use crate::escaping::Escaper;
    use crate::formatln;
    use crate::generators::generator::tests::UpdateGeneratorTest;
    use crate::generators::generator::tests::run_update_generator_tests;
    use crate::generators::generator::tests::standard_testcase_generator_test_suite;
    use crate::outcome::Outcome;
    use crate::parsers::parser::ParserType;
    use crate::test_expectation;
    use crate::testcase::TestCase;
    use crate::testcase::TestCaseError;

    #[test]
    fn test_update_generator() {
        let tests: &[(&str, UpdateGeneratorTest)] = &[
            (
                "simple_unchanged",
                UpdateGeneratorTest {
                    original_document: &([
                        "# This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "an expectation",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),
                    outcomes: vec![Outcome {
                        location: None,
                        output: ("an expectation\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!(
                                "equal",
                                "an expectation",
                                false,
                                false,
                                "an expectation"
                            )],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Ok(()),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "complex_unchanged",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "line * (glob+)",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),
                    outcomes: vec![Outcome {
                        location: None,
                        output: ("line 1\nline 2\nline 3\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!(
                                "glob",
                                "line *",
                                false,
                                true,
                                "line * (glob+)"
                            )],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Ok(()),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "updated_output_expectations",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "an expectation",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("new output\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![(0, formatln!("new output").as_bytes().to_vec())],
                            },
                        ]))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "updated_output_none_zero_exit_code",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "same output",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("same output\n", "", Some(10)).into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "same output")],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::InvalidExitCode {
                            actual: 10,
                            expected: 0,
                        }),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "updated_output_non_0_exit_no_output",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("", "", Some(10)).into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::InvalidExitCode {
                            actual: 10,
                            expected: 0,
                        }),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "updated_output_changed_exit_code",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "same output",
                        "[10]",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("same output\n", "", Some(20)).into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "same output")],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::InvalidExitCode {
                            actual: 20,
                            expected: 10,
                        }),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "use_minimal_amount_of_backticks",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "````scrut",
                        "$ the command",
                        "old output",
                        "````",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("new output\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![(0, formatln!("new output").as_bytes().to_vec())],
                            },
                        ]))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "adjust_to_right_amount_of_backticks",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "````scrut",
                        "$ the command",
                        "old output",
                        "````",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("````\nnew output\n````\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![
                                    (0, formatln!("````").as_bytes().to_vec()),
                                    (1, formatln!("new output").as_bytes().to_vec()),
                                    (2, formatln!("````").as_bytes().to_vec()),
                                ],
                            },
                        ]))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "per_document_config",
                UpdateGeneratorTest {
                    original_document: &([
                        "---",
                        "total_timeout: 2m 3s",
                        "defaults:",
                        "  timeout: 3m 4s",
                        "---",
                        "",
                        "This is a test",
                        "",
                        "```scrut",
                        "$ the command",
                        "old output",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("new output\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                            config: TestCaseConfig {
                                timeout: Some(Duration::from_secs(3 * 60 + 4)),
                                wait: Some(TestCaseWait {
                                    timeout: Duration::from_secs(4 * 60 + 5),
                                    path: Some(PathBuf::from("some-path")),
                                }),
                                ..Default::default()
                            },
                        },
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![(0, formatln!("new output").as_bytes().to_vec())],
                            },
                        ]))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "per_test_config",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut {timeout: 3m 4s, wait: {timeout: 4m 5s, path: some-path}}",
                        "$ the command",
                        "old output",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("new output\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                            config: TestCaseConfig {
                                timeout: Some(Duration::from_secs(3 * 60 + 4)),
                                wait: Some(TestCaseWait {
                                    timeout: Duration::from_secs(4 * 60 + 5),
                                    path: Some(PathBuf::from("some-path")),
                                }),
                                ..Default::default()
                            },
                        },
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![(0, formatln!("new output").as_bytes().to_vec())],
                            },
                        ]))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "comments_in_code",
                UpdateGeneratorTest {
                    original_document: &([
                        "This is a test",
                        "",
                        "```scrut",
                        "# some comment before",
                        "$ the command",
                        "old output",
                        "```",
                    ]
                    .join("\n")
                        + "\n"),

                    outcomes: vec![Outcome {
                        location: None,
                        output: ("new output\n", "").into(),
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                            ..Default::default()
                        },
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![(0, formatln!("new output").as_bytes().to_vec())],
                            },
                        ]))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }],
                },
            ),
            (
                "updated_only_testcases",
                UpdateGeneratorTest {
                    original_document: &([
                        "# this is a document",
                        "",
                        "there are many lines",
                        "",
                        "```rust",
                        "println!(\"including code\");",
                        "```",
                        "",
                        "## Then there is a test",
                        "",
                        "```scrut",
                        "$ the command 1",
                        "old output 1",
                        "```",
                        "",
                        "Followed by other stuff",
                        "",
                        "",
                        "",
                        "Followed by another test",
                        "",
                        "```scrut",
                        "$ the command 2",
                        "old output 2",
                        "```",
                        "",
                        "ultimately ending in stuff",
                    ]
                    .join("\n")
                        + "\n"),
                    outcomes: vec![
                        Outcome {
                            location: None,
                            output: ("new output ONE\n", "").into(),
                            testcase: TestCase {
                                title: "Then there is a test".to_string(),
                                shell_expression: "the command 1".to_string(),
                                expectations: vec![test_expectation!("equal", "old output 1")],
                                exit_code: None,
                                line_number: 234,
                                ..Default::default()
                            },
                            result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                                DiffLine::UnmatchedExpectation {
                                    index: 0,
                                    expectation: test_expectation!("equal", "old output 1"),
                                },
                                DiffLine::UnexpectedLines {
                                    lines: vec![(
                                        0,
                                        formatln!("new output ONE").as_bytes().to_vec(),
                                    )],
                                },
                            ]))),
                            escaping: Escaper::default(),
                            format: ParserType::Markdown,
                        },
                        Outcome {
                            location: None,
                            output: ("new output ZWEI\n", "").into(),
                            testcase: TestCase {
                                title: "Followed by another test".to_string(),
                                shell_expression: "the command 2".to_string(),
                                expectations: vec![test_expectation!("equal", "old output 2")],
                                exit_code: None,
                                line_number: 234,
                                ..Default::default()
                            },
                            result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                                DiffLine::UnmatchedExpectation {
                                    index: 0,
                                    expectation: test_expectation!("equal", "old output 2"),
                                },
                                DiffLine::UnexpectedLines {
                                    lines: vec![(
                                        0,
                                        formatln!("new output ZWEI").as_bytes().to_vec(),
                                    )],
                                },
                            ]))),
                            escaping: Escaper::default(),
                            format: ParserType::Markdown,
                        },
                    ],
                },
            ),
        ];

        let generator = MarkdownUpdateGenerator::default();
        run_update_generator_tests(generator, "markdown", tests);
    }

    #[test]
    fn test_testcase_generator() {
        let generator = MarkdownTestCaseGenerator::default();
        standard_testcase_generator_test_suite(generator, "markdown")
    }
}
