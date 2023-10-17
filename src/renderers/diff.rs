use std::cmp::Ordering;
use std::fmt::Display;

use anyhow::bail;
use anyhow::Result;

use super::renderer::ErrorRenderer;
use super::renderer::Renderer;
use crate::diff::Diff;
use crate::diff::DiffLine;
use crate::formatln;
use crate::newline::BytesNewline;
use crate::outcome::Outcome;
use crate::parsers::parser::ParserType;

/// Renderer that uses the traditional Diff render format
/// See: https://en.wikipedia.org/wiki/Diff
pub struct DiffRenderer {}

impl DiffRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DiffRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for DiffRenderer {
    fn render(&self, outcomes: &[&Outcome]) -> Result<String> {
        let mut output = String::new();
        let count_locations = outcomes
            .iter()
            .filter(|outcome| outcome.location.is_some())
            .count();
        let mut outcomes = outcomes.to_owned().to_vec();
        if count_locations > 0 {
            if count_locations != outcomes.len() {
                bail!("cannot render diff with some outcomes providing locations, but not all")
            }
            outcomes.sort_by(|a, b| {
                let result = a.location.cmp(&b.location);
                if result == Ordering::Equal {
                    a.testcase.line_number.cmp(&b.testcase.line_number)
                } else {
                    result
                }
            })
        }
        let mut last_location = None;
        for outcome in outcomes {
            match &outcome.result {
                Ok(_) => continue,
                Err(err) => {
                    if outcome.location != last_location {
                        if let Some(ref location) = outcome.location {
                            if last_location.is_some() {
                                output.push('\n');
                            }
                            last_location = outcome.location.to_owned();
                            output.push_str(&formatln!("--- {}", location));
                            output.push_str(&formatln!("+++ {}.new", location));
                        }
                    }
                    let rendered_error = self.render_error(err, outcome)?;
                    output.push_str(&rendered_error);
                }
            }
        }
        Ok(output)
    }
}

impl ErrorRenderer for DiffRenderer {
    /// Render result contains a change for the last line of the output, which
    /// would contain the exit code `[<exit-code>]`.
    /// The rest of the output is *not* attended, even if it is wrong.
    fn render_invalid_exit_code(
        &self,
        outcome: &Outcome,
        actual: i32,
        _expected: i32,
    ) -> Result<String> {
        let line_number = outcome.testcase.line_number
            + outcome.testcase.shell_expression_lines()
            + outcome.testcase.expectations_lines();
        let prefix = line_prefix(outcome);
        let title = join_multiline(&outcome.testcase.title, " * ");
        let mut output = String::new();
        output.push_str(
            &DiffHeader {
                old_start: line_number,
                old_length: outcome.testcase.exit_code.map_or(0, |_| 1),
                new_start: line_number,
                new_length: 1,
                kind: DiffHeaderKind::InvalidExitCode,
                title: &title,
            }
            .to_string(),
        );

        if let Some(exit_code) = outcome.testcase.exit_code {
            output.push_str(&format!("-{prefix}[{exit_code}]\n"));
        }
        output.push_str(&format!("+{prefix}[{actual}]\n"));
        Ok(output)
    }

    /// Renders the internal error, in a way that is NOT compatible with
    /// the unified diff syntax and will be rejected by `patch`. This is
    /// intentional - the error must be handled by a user.
    fn render_delegated_error(&self, outcome: &Outcome, err: &anyhow::Error) -> Result<String> {
        let title = join_multiline(&outcome.testcase.title, " * ");
        let mut output = String::new();
        output.push_str("# ---- INTERNAL ERROR ----\n");
        if let Some(ref location) = outcome.location {
            output.push_str(&format!("# PATH:  {location}\n"));
        }
        output.push_str(&format!("# TITLE: {title}\n"));
        let output_err = err
            .to_string()
            .lines()
            .map(|line| format!("# ERROR: {line}\n"))
            .collect::<Vec<_>>()
            .join("");
        output.push_str(&output_err);
        output.push_str("# ---- INTERNAL ERROR ----\n");
        Ok(output)
    }

    fn render_malformed_output(&self, outcome: &Outcome, diff: &Diff) -> Result<String> {
        UnifiedDiff::default().render(outcome, diff)
    }

    fn render_skipped(&self, _outcome: &Outcome) -> Result<String> {
        Ok("".into())
    }
}

#[derive(Default)]
struct UnifiedDiff {
    unmatched_start: Option<usize>,
    unmatched_lines: Vec<String>,
    unexpected_start: Option<usize>,
    unexpected_lines: Vec<(usize, String)>,
}

impl UnifiedDiff {
    fn flush(&mut self) {
        self.unmatched_start = None;
        self.unexpected_start = None;
        self.unmatched_lines = vec![];
        self.unexpected_lines = vec![];
    }

    fn render(&mut self, outcome: &Outcome, diff: &Diff) -> Result<String> {
        let line_number = outcome.testcase.line_number + outcome.testcase.shell_expression_lines();
        let title = join_multiline(&outcome.testcase.title, " * ");
        let prefix = line_prefix(outcome);
        let mut output = String::new();

        macro_rules! add_diff_hunk {
            () => {
                if self.unmatched_start.is_some() || self.unexpected_start.is_some() {
                    let (unmatched_start, unexpected_start) =
                        if let Some(unmatched_start) = self.unmatched_start {
                            (
                                unmatched_start,
                                self.unexpected_start.unwrap_or(unmatched_start),
                            )
                        } else {
                            let unexpected_start = self.unexpected_start.unwrap();
                            (unexpected_start, unexpected_start)
                        };
                    output.push_str(
                        &DiffHeader {
                            old_start: unmatched_start + line_number,
                            old_length: self.unmatched_lines.len(),
                            new_start: unexpected_start + line_number,
                            new_length: self.unexpected_lines.len(),
                            kind: DiffHeaderKind::MalformedOutput,
                            title: &title,
                        }
                        .to_string(),
                    );
                    self.unmatched_lines
                        .iter()
                        .for_each(|line| output.push_str(&format!("-{prefix}{line}\n")));
                    self.unexpected_lines
                        .iter()
                        .for_each(|line| output.push_str(&format!("+{}{}\n", prefix, &line.1)));
                    self.flush();
                }
            };
        }

        let mut expectation_index = 0;
        for line in &diff.lines {
            match line {
                DiffLine::MatchedExpectation {
                    index,
                    expectation: _,
                    lines: _,
                } => {
                    expectation_index = *index;
                    add_diff_hunk!();
                }
                DiffLine::UnmatchedExpectation { index, expectation } => {
                    expectation_index = *index;
                    if self.unmatched_start.is_none() {
                        self.unmatched_start = Some(*index);
                    }
                    self.unmatched_lines.push(expectation.original_string())
                }
                DiffLine::UnexpectedLines { lines } => {
                    if self.unexpected_start.is_none() {
                        self.unexpected_start = Some(expectation_index)
                    }
                    self.unexpected_lines.extend(
                        lines
                            .iter()
                            .map(|(i, l)| {
                                Ok((
                                    *i,
                                    String::from_utf8((l as &[u8]).trim_newlines().to_vec())?,
                                ))
                            })
                            .collect::<Result<Vec<_>>>()?,
                    );
                    if self.unmatched_start.is_some() {
                        add_diff_hunk!();
                    }
                }
            }
        }
        add_diff_hunk!();

        Ok(output)
    }
}

enum DiffHeaderKind {
    InvalidExitCode,
    MalformedOutput,
}

impl Display for DiffHeaderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DiffHeaderKind::InvalidExitCode => "invalid exit code",
                DiffHeaderKind::MalformedOutput => "malformed output",
            }
        )
    }
}

struct DiffHeader<'a> {
    old_start: usize,
    old_length: usize,
    new_start: usize,
    new_length: usize,
    title: &'a str,
    kind: DiffHeaderKind,
}

impl<'a> Display for DiffHeader<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@@ -{}{} +{}{} @@ {}: {}",
            self.old_start,
            length_suffix(self.old_length),
            self.new_start,
            length_suffix(self.new_length),
            self.kind,
            self.title,
        )
    }
}

fn length_suffix(len: usize) -> String {
    if len > 1 || len == 0 {
        format!(",{len}")
    } else {
        "".into()
    }
}

fn join_multiline(text: &str, sep: &str) -> String {
    text.lines().collect::<Vec<_>>().join(sep)
}

fn line_prefix(outcome: &Outcome) -> &'static str {
    match outcome.format {
        ParserType::Markdown => "",
        ParserType::Cram => "  ",
    }
}

#[cfg(test)]
mod tests {
    use super::DiffRenderer;
    use crate::diff::Diff;
    use crate::diff::DiffLine;
    use crate::escaping::Escaper;
    use crate::outcome::Outcome;
    use crate::parsers::parser::ParserType;
    use crate::renderers::renderer::Renderer;
    use crate::test_expectation;
    use crate::testcase::TestCase;
    use crate::testcase::TestCaseError;

    #[test]
    fn test_render_success() {
        let renderer = DiffRenderer::new();
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
        assert_eq!("", &rendered, "success results are not rendered");
    }

    #[test]
    fn test_internal_error() {
        let renderer = DiffRenderer::new();
        let rendered = renderer
            .render(&[&Outcome {
                output: ("the stdout", "the stderr", Some(222)).into(),
                testcase: TestCase {
                    title: "the title".into(),
                    shell_expression: "the command".into(),
                    expectations: vec![],
                    exit_code: Some(111),
                    line_number: 234,
                    ..Default::default()
                },
                location: Some("the location".into()),
                result: Err(TestCaseError::InternalError(anyhow::Error::msg(
                    "bad thing",
                ))),
                escaping: Escaper::default(),
                format: ParserType::Markdown,
            }])
            .expect("render succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_invalid_exit_code() {
        let renderer = DiffRenderer::new();

        [ParserType::Cram, ParserType::Markdown]
            .iter()
            .for_each(|parser_type| {
                let rendered = renderer
                    .render(&[&Outcome {
                        output: ("the stdout\n", "the stderr\n", Some(222)).into(),
                        testcase: TestCase {
                            title: "the title".into(),
                            shell_expression: "the command".into(),
                            expectations: vec![test_expectation!("the stdout")],
                            exit_code: Some(111),
                            line_number: 234,
                            ..Default::default()
                        },
                        location: Some("the location".into()),
                        result: Err(TestCaseError::InvalidExitCode {
                            actual: 222,
                            expected: 111,
                        }),
                        escaping: Escaper::default(),
                        format: *parser_type,
                    }])
                    .expect("render succeeds");
                insta::assert_snapshot!(format!("invalid_exit_code_{parser_type}"), rendered);
            });
    }

    #[test]
    fn test_malformed_output() {
        let renderer = DiffRenderer::new();
        let testcase = TestCase {
            title: "the title".into(),
            shell_expression: "the command".into(),
            expectations: vec![
                test_expectation!("expected line 1"),
                test_expectation!("expected line 2"),
                test_expectation!("expected line 3"),
            ],
            exit_code: None,
            line_number: 234,
            ..Default::default()
        };

        let tests = vec![
            (
                "missing",
                Diff::new(vec![DiffLine::UnmatchedExpectation {
                    index: 1,
                    expectation: testcase.expectations[1].clone(),
                }]),
            ),
            (
                "unexpected",
                Diff::new(vec![DiffLine::UnexpectedLines {
                    lines: vec![(2, "something else".as_bytes().to_vec())],
                }]),
            ),
            (
                "mismatch",
                Diff::new(vec![
                    DiffLine::UnmatchedExpectation {
                        index: 1,
                        expectation: testcase.expectations[1].clone(),
                    },
                    DiffLine::UnmatchedExpectation {
                        index: 2,
                        expectation: testcase.expectations[2].clone(),
                    },
                    DiffLine::UnexpectedLines {
                        lines: vec![
                            (2, "something line 1".as_bytes().to_vec()),
                            (3, "something line 2".as_bytes().to_vec()),
                        ],
                    },
                ]),
            ),
        ];

        [ParserType::Cram, ParserType::Markdown]
            .iter()
            .for_each(|parser_type| {
                tests.iter().for_each(|(name, diff)| {
                    let rendered = renderer
                        .render(&[&Outcome {
                            output: (
                                "expected line 1\nexpected line FAIL\nexpected line 3\n",
                                "the stderr",
                            )
                                .into(),
                            testcase: testcase.clone(),
                            location: Some("the location".into()),
                            result: Err(TestCaseError::MalformedOutput(diff.to_owned())),
                            escaping: Escaper::default(),
                            format: *parser_type,
                        }])
                        .expect("render succeeds");
                    insta::assert_snapshot!(
                        format!("malformed_output_{name}_{parser_type}"),
                        rendered
                    );
                });
            })
    }

    #[test]
    fn test_render() {
        let renderer = DiffRenderer::new();
        let testcase = TestCase {
            title: "the title".into(),
            shell_expression: "the command".into(),
            expectations: vec![
                test_expectation!("expected line 1"),
                test_expectation!("expected line 2"),
                test_expectation!("expected line 3"),
            ],
            exit_code: None,
            line_number: 234,
            ..Default::default()
        };

        let tests = vec![
            (
                "missing",
                Diff::new(vec![DiffLine::UnmatchedExpectation {
                    index: 1,
                    expectation: testcase.expectations[1].clone(),
                }]),
            ),
            (
                "unexpected",
                Diff::new(vec![DiffLine::UnexpectedLines {
                    lines: vec![(2, "something else".as_bytes().to_vec())],
                }]),
            ),
            (
                "mismatch",
                Diff::new(vec![
                    DiffLine::UnmatchedExpectation {
                        index: 1,
                        expectation: testcase.expectations[1].clone(),
                    },
                    DiffLine::UnmatchedExpectation {
                        index: 2,
                        expectation: testcase.expectations[2].clone(),
                    },
                    DiffLine::UnexpectedLines {
                        lines: vec![
                            (2, "something line 1".as_bytes().to_vec()),
                            (3, "something line 2".as_bytes().to_vec()),
                        ],
                    },
                ]),
            ),
        ];

        [ParserType::Cram, ParserType::Markdown]
            .iter()
            .for_each(|parser_type| {
                tests.iter().for_each(|(name, diff)| {
                    let mut testcase1 = testcase.clone();
                    testcase1.line_number = 10;
                    let mut testcase2 = testcase.clone();
                    testcase2.line_number = 20;
                    let outcomes = vec![
                        Outcome {
                            output: (
                                "expected line 1\nexpected line FAIL\nexpected line 3\n",
                                "the stderr",
                            )
                                .into(),
                            testcase: testcase2,
                            location: Some("location2".into()),
                            result: Err(TestCaseError::MalformedOutput(diff.to_owned())),
                            format: *parser_type,
                            escaping: Escaper::default(),
                        },
                        Outcome {
                            output: (
                                "expected line 1\nexpected line FAIL\nexpected line 3\n",
                                "the stderr",
                            )
                                .into(),
                            testcase: testcase1,
                            location: Some("location1".into()),
                            result: Err(TestCaseError::MalformedOutput(diff.to_owned())),
                            format: *parser_type,
                            escaping: Escaper::default(),
                        },
                    ];
                    let rendered = renderer
                        .render(&outcomes.iter().collect::<Vec<_>>())
                        .expect("render succeeds");
                    insta::assert_snapshot!(format!("render_{name}_{parser_type}"), rendered);
                });
            })
    }
}
