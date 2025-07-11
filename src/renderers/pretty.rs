/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use anyhow::Result;
use console::style;

use super::outcome::OutcomeHeader;
use super::renderer::ErrorRenderer;
use super::renderer::Renderer;
use crate::diff::Diff;
use crate::diff::DiffLine;
use crate::escaping::strip_colors;
use crate::formatln;
use crate::newline::BytesNewline;
use crate::newline::StringNewline;
use crate::outcome::Outcome;
use crate::testcase::TestCaseError;

pub const DEFAULT_SURROUNDING_LINES: usize = 5;
pub const DEFAULT_ABSOLUTE_LINE_NUMBERS: bool = false;
pub const DEFAULT_SUMMARIZE: bool = true;
pub const DEFAULT_MULTILINE_MATCHED_LINES: usize = 100;

/// Renders errors in a human readable way, that higlights the differences eper
/// test case.
#[derive(Default)]
pub struct PrettyMonochromeRenderer(PrettyColorRenderer);

impl PrettyMonochromeRenderer {
    pub fn new(color_renderer: PrettyColorRenderer) -> Self {
        Self(color_renderer)
    }
}

impl Renderer for PrettyMonochromeRenderer {
    fn render(&self, outcomes: &[&Outcome]) -> Result<String> {
        let rendered = self.0.render(outcomes)?;
        strip_colors(&rendered)
    }
}

/// Renders errors in a human readable way, that higlights the differences eper
/// test case. Uses colors!
pub struct PrettyColorRenderer {
    pub max_surrounding_lines: usize,
    pub absolute_line_numbers: bool,
    pub summarize: bool,
    pub max_multiline_matched_lines: usize,
}

impl PrettyColorRenderer {
    fn render_summary(&self, files: usize, ok: usize, errors: usize, ignored: usize) -> String {
        let summary = style("Result").underlined();
        let total = ok + errors + ignored;
        let tests = style(format!("{} testcase(s)", total)).bold();
        let mut succeeded = style(format!("{} succeeded", ok)).green();
        if ok > 0 {
            succeeded = succeeded.bold();
        }
        let mut failed = style(format!("{} failed", errors)).red();
        if errors > 0 {
            failed = failed.bold();
        }
        let mut skipped = style(format!("{} skipped", ignored)).yellow();
        if ignored > 0 {
            skipped = skipped.bold();
        }
        format!(
            "{}: {} document(s) with {}: {}, {} and {}\n",
            summary, files, tests, succeeded, failed, skipped,
        )
    }
}

impl Default for PrettyColorRenderer {
    fn default() -> Self {
        PrettyColorRenderer {
            max_surrounding_lines: DEFAULT_SURROUNDING_LINES,
            absolute_line_numbers: DEFAULT_ABSOLUTE_LINE_NUMBERS,
            summarize: DEFAULT_SUMMARIZE,
            max_multiline_matched_lines: DEFAULT_MULTILINE_MATCHED_LINES,
        }
    }
}

impl Renderer for PrettyColorRenderer {
    fn render(&self, outcomes: &[&Outcome]) -> Result<String> {
        let mut output = String::new();
        let mut count_errors = 0;
        let mut count_ok = 0;
        let mut count_skipped = 0;
        let mut locations = HashMap::new();

        for outcome in outcomes {
            if let Some(ref location) = outcome.location {
                locations.insert(location, true);
            }
            if let Err(ref err) = outcome.result {
                if matches!(err, TestCaseError::Skipped) {
                    count_skipped += 1;
                    continue;
                }
                count_errors += 1;
                output.push_str(&outcome.render_header()?);
                output.push_str(&self.render_error(err, outcome)?);
                output.push_str("\n\n");
            } else {
                count_ok += 1;
            }
        }

        if self.summarize {
            output.push_str(&self.render_summary(
                locations.len(),
                count_ok,
                count_errors,
                count_skipped,
            ));
        }
        Ok(output)
    }
}

impl ErrorRenderer for PrettyColorRenderer {
    fn render_invalid_exit_code(
        &self,
        outcome: &Outcome,
        actual: i32,
        expected: i32,
    ) -> Result<String> {
        let mut out = String::new();
        out.push_str(&formatln!("unexpected exit code"));
        out.push_str(&formatln!("  expected: {}", expected));
        out.push_str(&formatln!("  actual:   {}", actual));
        out.push_str(&formatln!(""));
        out.push_str(&outcome.output.to_error_string(&outcome.escaping));
        Ok(out)
    }

    fn render_delegated_error(&self, _outcome: &Outcome, err: &anyhow::Error) -> Result<String> {
        Ok(formatln!("error: {}", err))
    }

    fn render_malformed_output(&self, outcome: &Outcome, diff: &Diff) -> Result<String> {
        let mut output = String::new();
        let line_base = if self.absolute_line_numbers {
            outcome.testcase.line_number + outcome.testcase.shell_expression_lines() - 1
        } else {
            0
        };
        let decorator = Decorator::new(
            line_base
                + diff
                    .count_output_lines
                    .max(outcome.testcase.expectations.len()),
        );
        let mut last_error_index = None;
        let next_error_index = |index: usize| {
            diff.lines
                .iter()
                .skip(index)
                .position(|line| {
                    !matches!(
                        line,
                        DiffLine::MatchedExpectation {
                            index: _,
                            expectation: _,
                            lines: _,
                        }
                    )
                })
                .map(|v| v + index)
        };

        for (diff_index, line) in diff.lines.iter().enumerate() {
            match line {
                DiffLine::MatchedExpectation {
                    index,
                    expectation,
                    lines,
                } => {
                    let mut skip = true;
                    let mut first_skip = false;
                    if self.max_surrounding_lines > 0 {
                        if let Some(last_error_index) = last_error_index {
                            if last_error_index + self.max_surrounding_lines >= diff_index {
                                skip = false;
                            } else if last_error_index + self.max_surrounding_lines + 1
                                == diff_index
                            {
                                first_skip = true;
                            }
                        }
                        if let Some(next_error_index) = next_error_index(diff_index + 1) {
                            if diff_index + self.max_surrounding_lines >= next_error_index {
                                skip = false;
                            }
                        }
                    } else {
                        skip = false;
                    }
                    if !skip {
                        if !expectation.multiline || self.max_multiline_matched_lines <= 1 {
                            output.push_str(
                                &decorator
                                    .line(
                                        if expectation.multiline {
                                            Some(0)
                                        } else {
                                            Some(line_base + lines[0].0 + 1)
                                        },
                                        Some(line_base + index + 1),
                                        expectation.multiline,
                                        " ",
                                        &expectation.to_expression_string(&outcome.escaping),
                                    )
                                    .assure_newline(),
                            );
                        } else {
                            let mut half = 0;
                            let lines = if lines.len() > self.max_multiline_matched_lines {
                                // take lines from the top and the bottom and truncate the middle
                                half = ((self.max_multiline_matched_lines as f64) / 2.0).ceil()
                                    as usize;
                                let mut new_lines: Vec<(usize, Vec<u8>)> =
                                    Vec::with_capacity(self.max_multiline_matched_lines);
                                new_lines.extend(lines.iter().take(half).cloned());
                                new_lines.extend(
                                    lines
                                        .iter()
                                        .rev()
                                        .take(self.max_multiline_matched_lines - half)
                                        .rev()
                                        .cloned(),
                                );
                                new_lines
                            } else {
                                lines.to_vec()
                            };

                            let max_line_length =
                                lines.iter().map(|(_, line)| line.len()).max().unwrap_or(0);
                            for (output_index, (line_index, line)) in lines.into_iter().enumerate()
                            {
                                let mut line = outcome
                                    .escaping
                                    .escaped_printable((&line as &[u8]).trim_newlines())
                                    .to_string();

                                // append expectation as a "comment" infirst line
                                if output_index == 0 {
                                    line += &format!(
                                        "{}{} {}",
                                        " ".repeat(max_line_length - line.len() + 1),
                                        style("//").magenta(),
                                        style(expectation.to_expression_string(&outcome.escaping))
                                            .magenta()
                                            .bold()
                                    );
                                }
                                output.push_str(
                                    &decorator
                                        .line(
                                            Some(line_base + line_index + 1),
                                            if output_index == 0 {
                                                Some(line_base + index + 1)
                                            } else {
                                                None
                                            },
                                            expectation.multiline,
                                            " ",
                                            &line,
                                        )
                                        .assure_newline(),
                                );

                                // insert "..." if half of the lines have been added
                                if half > 0 && output_index + 1 == half {
                                    output.push_str(
                                        &decorator
                                            .line(None, None, expectation.multiline, "…", "")
                                            .assure_newline(),
                                    );
                                }
                            }
                        }
                    } else if first_skip {
                        output.push_str(&"...".assure_newline());
                    }
                }
                DiffLine::UnmatchedExpectation { index, expectation } => {
                    last_error_index = Some(diff_index);
                    let content = expectation
                        .to_expression_string(&outcome.escaping)
                        .higlight_tailing_spaces();
                    output.push_str(
                        &decorator
                            .line(
                                None,
                                Some(line_base + index + 1),
                                expectation.multiline,
                                "-",
                                &content,
                            )
                            .assure_newline(),
                    )
                }
                DiffLine::UnexpectedLines { lines } => {
                    lines.iter().for_each(|(line_index, line)| {
                        let eol = (line.as_ref() as &[u8]).ends_in_newline();
                        let line = if !eol {
                            let mut line = line.clone();
                            line.extend(b" (no-eol)");
                            line
                        } else {
                            line.to_owned()
                        };
                        let line = outcome
                            .escaping
                            .escaped_expectation(&line)
                            .higlight_tailing_spaces();
                        last_error_index = Some(diff_index);
                        output.push_str(
                            &decorator
                                .line(Some(line_base + line_index + 1), None, false, "+", &line)
                                .assure_newline(),
                        )
                    })
                }
            }
        }

        Ok(output)
    }

    fn render_timeout(&self, outcome: &Outcome) -> Result<String> {
        let mut out = String::new();
        out.push_str(&formatln!("timeout in execution"));
        out.push_str(&formatln!(""));
        out.push_str(&outcome.output.to_error_string(&outcome.escaping));
        Ok(out)
    }

    fn render_skipped(&self, _outcome: &Outcome) -> Result<String> {
        Ok("".into())
    }
}

trait TailingSpacesHighlighter {
    fn higlight_tailing_spaces(&self) -> String;
}

impl<T: AsRef<str>> TailingSpacesHighlighter for T {
    fn higlight_tailing_spaces(&self) -> String {
        let input = self.as_ref();
        let index = space_start_index(input);
        if index < input.len() {
            let prefix = &input[0..index];
            let suffix = style(render_spaces(&input[index..])).magenta().bold();
            format!("{prefix}{suffix}")
        } else {
            input.to_string()
        }
    }
}

fn space_start_index(input: &str) -> usize {
    for (i, ch) in input.chars().rev().enumerate() {
        if !ch.is_whitespace() {
            return input.len() - i;
        }
    }
    0
}

fn render_spaces(spaces: &str) -> String {
    let mut visible = String::new();
    for ch in spaces.chars() {
        visible.push(match ch {
            '\t' => '↦',
            ' ' => '⎵',
            _ => '⍰',
        });
    }
    visible
}

struct Decorator(usize);

impl Decorator {
    fn new(max_lines: usize) -> Self {
        Self(max_lines.to_string().len())
    }

    fn output_line_number(&self, num: Option<usize>) -> String {
        match num {
            None => " ".repeat(self.0),
            Some(num) => {
                let (num, prefix) = if num == 0 {
                    ("".into(), "+")
                } else {
                    (num.to_string(), " ")
                };
                let prefix = prefix.repeat(self.0 - num.len());
                format!("{}{}", prefix, num)
            }
        }
    }

    fn expectation_line_number(&self, num: Option<usize>, multiline: bool) -> String {
        let sign = if multiline { "+" } else { " " };
        format!("{}{}", self.output_line_number(num), sign)
    }

    fn line(
        &self,
        line_number: Option<usize>,
        expectation_number: Option<usize>,
        multiline: bool,
        symbol: &str,
        content: &str,
    ) -> String {
        let color = match symbol {
            "+" => |s: &str| style(s).green().bold().to_string(),
            "-" => |s: &str| style(s).red().bold().to_string(),
            _ => |s: &str| style(s).white().to_string(),
        };
        let line_color = match symbol {
            "+" => |s: &str| style(s).green().to_string(),
            "-" => |s: &str| style(s).red().to_string(),
            _ => |s: &str| s.to_string(),
        };
        style(format!(
            "{} {}  | {} {}",
            line_color(&self.expectation_line_number(expectation_number, multiline)),
            line_color(&self.output_line_number(line_number)),
            color(symbol),
            color(content)
        ))
        .bright()
        .black()
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::PrettyColorRenderer;
    use super::PrettyMonochromeRenderer;
    use crate::bformatln;
    use crate::diff::Diff;
    use crate::diff::DiffLine;
    use crate::escaping::Escaper;
    use crate::formatln;
    use crate::outcome::Outcome;
    use crate::parsers::parser::ParserType;
    use crate::renderers::renderer::Renderer;
    use crate::test_expectation;
    use crate::testcase::TestCase;
    use crate::testcase::TestCaseError;

    fn new_test_renderer() -> PrettyMonochromeRenderer {
        PrettyMonochromeRenderer::new(PrettyColorRenderer {
            max_surrounding_lines: 0,
            absolute_line_numbers: false,
            summarize: true,
            ..Default::default()
        })
    }

    #[test]
    fn test_render_success() {
        let renderer = new_test_renderer();
        let rendered = renderer
            .render(&[&Outcome {
                output: ("the stdout", "the stderr").into(),
                testcase: TestCase {
                    title: "the title".to_string(),
                    shell_expression: "the command".to_string(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                location: Some("the location".to_string()),
                result: Ok(()),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render succeeds");
        assert_eq!(
            "Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped\n",
            &rendered,
            "success results are not rendered",
        );
    }

    #[test]
    fn test_render_multiline() {
        let renderer = new_test_renderer();
        let rendered = renderer
            .render(&[&Outcome {
                output: ("the stdout", "the stderr").into(),
                testcase: TestCase {
                    title: "the title \\\nnext line \\\nlast line".into(),
                    shell_expression: "the command \\\nnext line \\\nlast line".into(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                location: Some("the location \\\nnext line \\\nlast line".into()),
                result: Err(TestCaseError::InvalidExitCode {
                    actual: 123,
                    expected: 234,
                }),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_render_allows_for_utf8() {
        let renderer = new_test_renderer();
        let rendered = renderer
            .render(&[&Outcome {
                output: ("the stdout", "the stderr").into(),
                testcase: TestCase {
                    title: "the title".to_string(),
                    shell_expression: "the \x1b[1mcommand\x1b[0m 🥳".to_string(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                location: Some("the location".to_string()),
                result: Err(TestCaseError::InvalidExitCode {
                    actual: 123,
                    expected: 234,
                }),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_render_invalid_exit_code() {
        struct Test {
            absolute_line_numbers: bool,
            location: Option<String>,
        }
        let tests = [
            Test {
                absolute_line_numbers: false,
                location: None,
            },
            Test {
                absolute_line_numbers: true,
                location: None,
            },
            Test {
                absolute_line_numbers: false,
                location: Some("path/location.md".into()),
            },
            Test {
                absolute_line_numbers: true,
                location: Some("path/location.md".into()),
            },
        ];
        tests.iter().for_each(|test| {
            let renderer = PrettyMonochromeRenderer::new(PrettyColorRenderer {
                max_surrounding_lines: 0,
                absolute_line_numbers: test.absolute_line_numbers,
                summarize: true,
                ..Default::default()
            });
            let rendered = renderer
                .render(&[&Outcome {
                    output: ("the stdout", "the stderr").into(),
                    testcase: TestCase {
                        title: "the title".to_string(),
                        shell_expression: "the command".to_string(),
                        expectations: vec![],
                        exit_code: None,
                        line_number: 234,
                        ..Default::default()
                    },
                    location: test.location.clone(),
                    result: Err(TestCaseError::InvalidExitCode {
                        actual: 123,
                        expected: 234,
                    }),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                }])
                .expect("render succeeds");
            insta::assert_snapshot!(
                format!(
                    "render_invalid_exit_code_absolute={:?},location={}",
                    test.absolute_line_numbers,
                    test.location.is_some(),
                ),
                rendered
            );
        })
    }

    #[test]
    fn test_render_internal_error() {
        let renderer = new_test_renderer();
        let rendered = renderer
            .render(&[&Outcome {
                location: None,
                output: ("the stdout", "the stderr", Some(123)).into(),
                testcase: TestCase {
                    title: "the title".to_string(),
                    shell_expression: "the command".to_string(),
                    expectations: vec![],
                    exit_code: None,
                    line_number: 234,
                    ..Default::default()
                },
                result: Err(TestCaseError::InternalError(anyhow!("something failed"))),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render does not fail");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_render_malformed_output() {
        let renderer = new_test_renderer();
        let testcase = TestCase {
            title: "the title".to_string(),
            shell_expression: "the command".to_string(),
            expectations: vec![
                test_expectation!("equal", "matched", false, false),
                test_expectation!("equal", "unmatched", false, false),
                test_expectation!("equal", "unused1", false, false),
                test_expectation!("equal", "unused2", false, false),
            ],
            exit_code: None,
            line_number: 234,
            ..Default::default()
        };
        let rendered = renderer
            .render(&[&Outcome {
                location: None,
                output: ("matched\nno match 1\nno match 2\n", "", Some(123)).into(),
                testcase: testcase.clone(),
                result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                    DiffLine::MatchedExpectation {
                        index: 0,
                        expectation: testcase.expectations[0].clone(),
                        lines: vec![(0, bformatln!("matched"))],
                    },
                    DiffLine::UnmatchedExpectation {
                        index: 1,
                        expectation: testcase.expectations[1].clone(),
                    },
                    DiffLine::UnexpectedLines {
                        lines: vec![(1, bformatln!("no match 1")), (2, bformatln!("no match 2"))],
                    },
                ]))),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render does not fail");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_render_malformed_output_multiple_lines() {
        let renderer = PrettyMonochromeRenderer::new(PrettyColorRenderer {
            max_surrounding_lines: 10,
            absolute_line_numbers: false,
            summarize: true,
            ..Default::default()
        });
        let testcase = TestCase {
            title: "the title".to_string(),
            shell_expression: "the command".to_string(),
            expectations: vec![
                test_expectation!("equal", "foo", false, true),
                test_expectation!("equal", "bar", false, false),
                test_expectation!("equal", "baz", false, false),
            ],
            exit_code: None,
            line_number: 234,
            ..Default::default()
        };
        let rendered = renderer
            .render(&[&Outcome {
                location: None,
                output: ("foo\nfoo\nfoo\nbar\n", "").into(),
                testcase: testcase.clone(),
                result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                    DiffLine::MatchedExpectation {
                        index: 0,
                        expectation: testcase.expectations[0].clone(),
                        lines: vec![
                            (0, bformatln!("foo")),
                            (1, bformatln!("foo")),
                            (2, bformatln!("foo")),
                        ],
                    },
                    DiffLine::MatchedExpectation {
                        index: 1,
                        expectation: testcase.expectations[1].clone(),
                        lines: vec![(3, bformatln!("bar"))],
                    },
                    DiffLine::UnmatchedExpectation {
                        index: 2,
                        expectation: testcase.expectations[2].clone(),
                    },
                ]))),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render does not fail");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_render_malformed_output_surrounding_lines() {
        let mut expectations = vec![];
        let mut diff = vec![];
        for i in 1..11 {
            expectations.push(test_expectation!(
                "equal",
                &format!("line match {}", i),
                false,
                false
            ));
            diff.push(DiffLine::MatchedExpectation {
                index: i - 1,
                expectation: expectations[i - 1].clone(),
                lines: vec![(i - 1, bformatln!("line match {}", i))],
            });
        }

        expectations.push(test_expectation!("equal", "line NOT match", false, false));
        diff.push(DiffLine::UnmatchedExpectation {
            index: 10,
            expectation: expectations[10].clone(),
        });
        diff.push(DiffLine::UnexpectedLines {
            lines: vec![(10, bformatln!("actual line"))],
        });

        for i in 12..17 {
            expectations.push(test_expectation!(
                "equal",
                &format!("line match {}", i),
                false,
                false
            ));
            diff.push(DiffLine::MatchedExpectation {
                index: i - 1,
                expectation: expectations[i - 1].clone(),
                lines: vec![(i - 1, bformatln!("line match {}", i))],
            });
        }

        expectations.push(test_expectation!("equal", "line NOT match", false, false));
        diff.push(DiffLine::UnmatchedExpectation {
            index: 16,
            expectation: expectations[16].clone(),
        });
        diff.push(DiffLine::UnexpectedLines {
            lines: vec![(16, bformatln!("actual line"))],
        });

        for i in 18..22 {
            expectations.push(test_expectation!(
                "equal",
                &format!("line match {}", i),
                false,
                false
            ));
            diff.push(DiffLine::MatchedExpectation {
                index: i - 1,
                expectation: expectations[i - 1].clone(),
                lines: vec![(i - 1, bformatln!("line match {}", i))],
            });
        }

        for surrounding in 1..8 {
            let renderer = PrettyMonochromeRenderer::new(PrettyColorRenderer {
                max_surrounding_lines: surrounding,
                absolute_line_numbers: false,
                summarize: true,
                ..Default::default()
            });
            let testcase = TestCase {
                title: "the title".to_string(),
                shell_expression: "the command".to_string(),
                expectations: expectations.clone(),
                exit_code: None,
                line_number: 234,
                ..Default::default()
            };

            let rendered = renderer
                .render(&[&Outcome {
                    location: None,
                    output: ("matched\nno match 1\nno match 2\n", "", Some(123)).into(),
                    testcase,
                    result: Err(TestCaseError::MalformedOutput(Diff::new(diff.clone()))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                }])
                .expect("render does not fail");
            insta::assert_snapshot!(
                format!("test_render_only_matches_surrounding_error={}", surrounding),
                rendered
            );
        }
    }

    #[test]
    fn test_render_malformed_output_with_no_expectations_but_output() {
        let renderer = new_test_renderer();
        let testcase = TestCase {
            title: "the title".to_string(),
            shell_expression: "the command".to_string(),
            expectations: vec![],
            exit_code: None,
            line_number: 234,
            ..Default::default()
        };
        let rendered = renderer
            .render(&[&Outcome {
                location: None,
                output: (
                    &(0..=10)
                        .map(|index| formatln!("no match {}", index + 1))
                        .collect::<Vec<_>>()
                        .join(""),
                    "",
                    Some(123),
                )
                    .into(),
                testcase,
                result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                    DiffLine::UnexpectedLines {
                        lines: (0..=10)
                            .map(|index| (index, bformatln!("no match {}", index + 1)))
                            .collect(),
                    },
                ]))),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render does not fail");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_render_absolute_line_numbers() {
        let mut expectations = (1..21)
            .map(|n| test_expectation!("equal", &format!("matching {}", n)))
            .collect::<Vec<_>>();
        expectations.extend(
            (1..6)
                .map(|n| test_expectation!("equal", &format!("failing {}", 20 + n)))
                .collect::<Vec<_>>(),
        );
        expectations.extend(
            (26..46)
                .map(|n| test_expectation!("equal", &format!("matching {}", n)))
                .collect::<Vec<_>>(),
        );

        let lines = expectations
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if e.original_string().contains("failing") {
                    DiffLine::UnmatchedExpectation {
                        index: i,
                        expectation: e.clone(),
                    }
                } else {
                    DiffLine::MatchedExpectation {
                        index: i,
                        expectation: e.clone(),
                        lines: vec![(i, bformatln!("matched"))],
                    }
                }
            })
            .collect::<Vec<_>>();

        let testcase = TestCase {
            title: "the title".to_string(),
            shell_expression: "the command".to_string(),
            expectations,
            exit_code: None,
            line_number: 90,
            ..Default::default()
        };

        [false, true].iter().for_each(|absolute_numbers| {
            let renderer = PrettyMonochromeRenderer::new(PrettyColorRenderer {
                max_surrounding_lines: 0,
                absolute_line_numbers: *absolute_numbers,
                summarize: true,
                ..Default::default()
            });
            let rendered = renderer
                .render(&[&Outcome {
                    location: None,
                    output: ("matched\nno match 1\nno match 2\n", "", Some(123)).into(),
                    testcase: testcase.clone(),
                    result: Err(TestCaseError::MalformedOutput(Diff::new(lines.clone()))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                }])
                .expect("render does not fail");
            insta::assert_snapshot!(
                format!("test_render_absolute_line_numbers={:?}", *absolute_numbers),
                rendered
            );
        })
    }

    #[test]
    fn test_render_multiline_match_excerpt() {
        let output = format!("bar\n{}baz", "foo\n".repeat(12));
        let expectations = vec![test_expectation!("glob", "foo*", false, true)];
        let lines = vec![
            DiffLine::UnexpectedLines {
                lines: vec![(0, bformatln!("bar"))],
            },
            DiffLine::MatchedExpectation {
                index: 1,
                expectation: expectations[0].clone(),
                lines: (1..13)
                    .map(|i| (i, bformatln!("foo{}", i)))
                    .collect::<Vec<_>>(),
            },
            DiffLine::UnexpectedLines {
                lines: vec![(13, bformatln!("bar"))],
            },
        ];

        let testcase = TestCase {
            title: "the title".to_string(),
            shell_expression: "the command".to_string(),
            expectations,
            exit_code: None,
            line_number: 10,
            ..Default::default()
        };

        [12, 6, 3, 1]
            .into_iter()
            .for_each(|max_multiline_matched_lines| {
                let renderer = PrettyMonochromeRenderer::new(PrettyColorRenderer {
                    max_surrounding_lines: 0,
                    absolute_line_numbers: false,
                    summarize: false,
                    max_multiline_matched_lines: max_multiline_matched_lines as usize,
                });
                let rendered = renderer
                    .render(&[&Outcome {
                        location: None,
                        output: (&output, "", Some(123)).into(),
                        testcase: testcase.clone(),
                        result: Err(TestCaseError::MalformedOutput(Diff::new(lines.clone()))),
                        escaping: Escaper::default(),
                        format: ParserType::Markdown,
                    }])
                    .expect("render does not fail");
                insta::assert_snapshot!(
                    format!(
                        "test_render_multiline_match_excerpt={:?}",
                        max_multiline_matched_lines
                    ),
                    rendered
                );
            })
    }
}
