/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Debug;

use anyhow::Result;
use serde::ser::SerializeMap;
use serde::Serialize;

use crate::expectation::Expectation;
use crate::lossy_string;
use crate::newline::SplitLinesByNewline;

/// Compares [`crate::output::Output`]s of [`crate::executors::executor::Executor`]
/// with a list of [`crate::expectation::Expectation`]s to find out if there
/// are any differences, which are represented as:
///
/// - [`DiffLine::UnmatchedExpectation`], which are expectations that do not
///   match at the position where they are.
/// - [`DiffLine::UnexpectedLines`], which lines of output, that for which no
///   expectation is available that matches them
///
/// All other cases, where the expectation matches one or multiple lines,
/// are instances of [`DiffLine::MatchedExpectation`]
///
/// ```
/// use scrut::bformatln;
/// use scrut::blines;
/// use scrut::diff::DiffLine;
/// use scrut::diff::DiffTool;
/// use scrut::expectation::Expectation;
/// use scrut::expectation::ExpectationMaker;
/// use scrut::rules::registry::RuleRegistry;
///
/// let maker = ExpectationMaker::new(RuleRegistry::default());
/// let expectations = vec![
///     maker.parse("foo1").unwrap(),
///     maker.parse("bar").unwrap(),
///     maker.parse("baz").unwrap(),
/// ];
/// let diff_tool = DiffTool::new(expectations.clone());
/// let diff = diff_tool
///     .diff(&blines!("bla", "foo1", "foo2", "foo3", "bar"))
///     .expect("no error");
///
/// assert_eq!(
///     vec![
///         DiffLine::UnexpectedLines {
///             lines: vec![(0, bformatln!("bla"))]
///         },
///         DiffLine::MatchedExpectation {
///             index: 0,
///             expectation: expectations[0].clone(),
///             lines: vec![(1, bformatln!("foo1"))]
///         },
///         DiffLine::UnexpectedLines {
///             lines: vec![(2, bformatln!("foo2")), (3, bformatln!("foo3")),]
///         },
///         DiffLine::MatchedExpectation {
///             index: 1,
///             expectation: expectations[1].clone(),
///             lines: vec![(4, bformatln!("bar"))]
///         },
///         DiffLine::UnmatchedExpectation {
///             index: 2,
///             expectation: expectations[2].clone(),
///         },
///     ],
///     diff.lines
/// );
/// ```
pub struct DiffTool {
    expectations: Vec<Expectation>,
}

impl DiffTool {
    /// Construct from list of expectations
    pub fn new(expectations: Vec<Expectation>) -> Self {
        Self { expectations }
    }

    /// Compares output with expectations and returns line-wise results that
    /// describe whether and which expectations matched, did not match, were
    /// not used and which lines were unexpected
    pub fn diff(&self, output: &[u8]) -> Result<Diff> {
        let lines = output.split_at_newline();
        let to_output_list = |i| -> (usize, Vec<u8>) { (i, lines[i].to_owned()) };
        let mut expectation_index = 0;
        let mut line_index = 0;
        let mut diffs = vec![];
        let mut match_start = None;

        // iterate all output expectations and all lines of (actual) output
        // in one loop until reaching the end of either one
        while expectation_index < self.expectations.len() && line_index < lines.len() {
            // with a line and an expectation ..
            let expectation = &self.expectations[expectation_index];
            let next_expectation = self.expectations.get(expectation_index + 1);
            let line = lines[line_index];

            // .. that matches the line
            if expectation.matches(line) {
                // .. and is multiline -> keep going to next line(s)
                if expectation.multiline {
                    // .. unless next expectation is not multiline (not greedy) AND matches, then
                    // favor the more precise expectation and end the multiline run
                    if let Some(next_expectation) = next_expectation {
                        if (expectation.optional || match_start.is_some())
                            && next_expectation.matches(line)
                        {
                            // make sure to note the previous multiline expectation
                            if let Some(match_start_index) = match_start {
                                diffs.push(DiffLine::MatchedExpectation {
                                    index: expectation_index,
                                    expectation: expectation.to_owned(),
                                    lines: (match_start_index..line_index)
                                        .map(to_output_list)
                                        .collect(),
                                });
                            }

                            // and then assure the next expectation is selected
                            expectation_index += 1;
                            match_start = None;
                            continue;
                        }
                    }

                    // otherwise make sure to log the starting line of the multiline match
                    if match_start.is_none() {
                        match_start = Some(line_index);
                    }

                    // .. and proceed the multiline match in the next line
                    line_index += 1;
                    continue;
                }

                // .. so note the match and go to next line with next expectation
                diffs.push(DiffLine::MatchedExpectation {
                    index: expectation_index,
                    expectation: expectation.to_owned(),
                    lines: vec![(line_index, line.to_owned())],
                });
                line_index += 1;
                expectation_index += 1;
                continue;
            }

            // .. that does not match this line, but did for (multiple?) lines
            //    before: make sure to log those lines the match and attempt the
            //    next expectation for the current line
            if let Some(match_start_index) = match_start {
                diffs.push(DiffLine::MatchedExpectation {
                    index: expectation_index,
                    expectation: expectation.to_owned(),
                    lines: (match_start_index..line_index)
                        .map(to_output_list)
                        .collect(),
                });
                match_start = None;
                expectation_index += 1;
                continue;
            }
            match_start = None;

            // .. that does not match the current line, so ..
            //   .. let find whatever is closer (if any):
            match self.peek_match(line_index, &lines, expectation_index) {
                //     .. the next matching expectation for the current line
                PeekMatch::NextExpectation(next_expectation_index) => {
                    // .. note down not matching of all intermediate expectations
                    //    assuming it is not optional ..
                    (expectation_index..next_expectation_index)
                        .filter(|index| !self.expectations[*index].optional)
                        .for_each(|index| {
                            diffs.push(DiffLine::UnmatchedExpectation {
                                index,
                                expectation: self.expectations[index].clone(),
                            });
                        });

                    // .. then continue with matching expectation
                    expectation_index = next_expectation_index;
                }

                //     .. the next matching line for the current expectation
                PeekMatch::NextLine(next_line_index) => {
                    // .. note down not matching of all intermediate lines ..
                    diffs.push(DiffLine::UnexpectedLines {
                        lines: (line_index..next_line_index).map(to_output_list).collect(),
                    });

                    // .. then continue with matching line ..
                    line_index = next_line_index;
                }

                //     .. neither
                PeekMatch::None => {
                    // .. give up and go to next expectation (for the next line) and try again
                    if !expectation.optional {
                        diffs.push(DiffLine::UnmatchedExpectation {
                            index: expectation_index,
                            expectation: expectation.to_owned(),
                        });
                    }
                    expectation_index += 1;
                }
            }
        }

        // .. ending in a multiline expectation?
        if let Some(match_start) = match_start {
            diffs.push(DiffLine::MatchedExpectation {
                index: expectation_index,
                expectation: self.expectations[expectation_index].to_owned(),
                lines: (match_start..line_index).map(to_output_list).collect(),
            });
            expectation_index += 1;
        }

        // .. having unused expectations?
        if expectation_index < self.expectations.len() {
            (expectation_index..self.expectations.len())
                .filter(|index| !self.expectations[*index].optional)
                .for_each(|index| {
                    diffs.push(DiffLine::UnmatchedExpectation {
                        index,
                        expectation: self.expectations[index].to_owned(),
                    })
                });
        }

        // .. having any unvisited lines?
        if line_index < lines.len() {
            diffs.push(DiffLine::UnexpectedLines {
                lines: (line_index..lines.len()).map(to_output_list).collect(),
            });
        }

        Ok(Diff::new(diffs))
    }

    /// Returns either the index of the index of the next matching expectation
    /// for the current line or if there is none, then the next index of the
    /// line matching the current expectation - or none, if that doesn't exist
    /// either
    fn peek_match(
        &self,
        current_line_index: usize,
        lines: &[&[u8]],
        current_expectation_index: usize,
    ) -> PeekMatch {
        // attempt finding an expectation that matches the current line first
        let expectation_index = self
            .peek_matching_expectation(lines[current_line_index], current_expectation_index + 1);
        if let Some(expectation_index) = expectation_index {
            return PeekMatch::NextExpectation(expectation_index);
        }

        // attempt fallback to finding line that matches current expectation instead
        let line_index = self.peek_matching_line(
            &self.expectations[current_expectation_index],
            current_line_index + 1,
            lines,
        );
        if let Some(line_index) = line_index {
            return PeekMatch::NextLine(line_index);
        }

        PeekMatch::None
    }

    /// Returns the index of the future line that matches the given expectation, if any does
    fn peek_matching_line(
        &self,
        expectation: &Expectation,
        start_line_index: usize,
        lines: &[&[u8]],
    ) -> Option<usize> {
        lines
            .iter()
            .skip(start_line_index)
            .position(|line| expectation.matches(line))
            .map(|position| position + start_line_index)
    }

    /// Returns the index of the future expectation that matches the given line, if any does
    fn peek_matching_expectation(
        &self,
        line: &[u8],
        start_expectation_index: usize,
    ) -> Option<usize> {
        self.expectations
            .iter()
            .skip(start_expectation_index)
            .position(|expectation| expectation.matches(line))
            .map(|position| position + start_expectation_index)
    }
}

/// Enumerate the kind of peeked (future) match that was found
enum PeekMatch {
    /// A future expectation matchers the current line
    NextExpectation(usize),

    /// A future line matches the current expectation
    NextLine(usize),

    /// No line matches the current expectation nor does any expectation
    /// match the current line
    None,
}

/// The result of a `[DiffTool::diff]` operation, containing all the
/// output [`DiffLine`]s of the comparison
#[derive(Clone, PartialEq, Serialize)]
pub struct Diff {
    pub lines: Vec<DiffLine>,

    #[serde(skip)]
    pub count_matched: usize,

    #[serde(skip)]
    pub count_unmatched: usize,

    #[serde(skip)]
    pub count_output_lines: usize,
}

impl Diff {
    pub fn new(lines: Vec<DiffLine>) -> Self {
        let (mut count_matched, mut count_unmatched, mut count_output_lines) = (0, 0, 0);
        lines.iter().for_each(|line| match line {
            DiffLine::MatchedExpectation {
                index: _,
                expectation: _,
                lines,
            } => {
                count_matched += 1;
                count_output_lines += lines.len();
            }
            DiffLine::UnmatchedExpectation {
                index: _,
                expectation: _,
            } => {
                count_unmatched += 1;
            }
            DiffLine::UnexpectedLines { lines } => count_output_lines += lines.len(),
        });
        Self {
            lines,
            count_matched,
            count_unmatched,
            count_output_lines,
        }
    }

    /// Whether there are any differences in the result, i.e. not all lines
    /// are [`DiffLine::MatchedExpectation`]s
    pub fn has_differences(&self) -> bool {
        self.lines.iter().any(|line| {
            !matches!(
                line,
                DiffLine::MatchedExpectation {
                    index: _,
                    expectation: _,
                    lines: _,
                }
            )
        })
    }
}

impl Debug for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rendered = String::new();
        let (mut count_matched, mut count_unmatched, mut count_unexpected) = (0, 0, 0);
        for line in &self.lines {
            match &line {
                DiffLine::MatchedExpectation {
                    index: _,
                    expectation: _,
                    lines,
                } => count_matched += lines.len(),
                DiffLine::UnmatchedExpectation {
                    index: _,
                    expectation: _,
                } => count_unmatched += 1,
                DiffLine::UnexpectedLines { lines } => count_unexpected += lines.len(),
            }
            rendered.push_str(&format!("{line:?}"));
        }
        writeln!(
            f,
            "[matched: {count_matched}, unmatched: {count_unmatched}, unexpected: {count_unexpected}]",
        )?;
        write!(f, "{rendered}")?;
        Ok(())
    }
}

// TODO: consider rename DiffItem.. or ??
/// A "line" of diff output
#[derive(Clone, PartialEq)]
pub enum DiffLine {
    /// A Match describes a valid line
    MatchedExpectation {
        /// The index within the list of expectations
        index: usize,

        /// The expectation that matched
        expectation: Expectation,

        /// The line(s) of output that was were matched and their index
        lines: Vec<(usize, Vec<u8>)>,
    },

    /// A an expectations (may be optional?) that was not matched, hence skipped
    UnmatchedExpectation {
        /// The index within the list of expectations
        index: usize,

        /// The expectation that was not matching
        expectation: Expectation,
    },

    /// A set of intermediate lines for which no expectation could be found
    UnexpectedLines {
        /// The line(s) of output without expectation match
        lines: Vec<(usize, Vec<u8>)>,
    },
}

impl Debug for DiffLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MatchedExpectation {
                index,
                expectation,
                lines,
            } => {
                writeln!(f, "{:04}      | = {}", index + 1, expectation)?;
                for (index, line) in lines {
                    write!(f, "     {:04} | = {}", index + 1, lossy_string!(line))?;
                }
                Ok(())
            }
            Self::UnmatchedExpectation { index, expectation } => {
                writeln!(f, "{:04}      | - {}", index + 1, expectation)
            }
            Self::UnexpectedLines { lines } => {
                for (index, line) in lines {
                    write!(f, "     {:04} | + {}", index + 1, lossy_string!(line))?;
                }
                Ok(())
            }
        }
    }
}

impl Serialize for DiffLine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DiffLine::MatchedExpectation {
                index,
                expectation,
                lines,
            } => {
                let mut variant = serializer.serialize_map(Some(4))?;
                variant.serialize_entry("kind", "matched_expectation")?;
                variant.serialize_entry("index", index)?;
                variant.serialize_entry("expectation", &expectation)?;
                variant.serialize_entry("lines", &lines_to_strings(lines))?;
                variant.end()
            }
            DiffLine::UnmatchedExpectation { index, expectation } => {
                let mut variant = serializer.serialize_map(Some(3))?;
                variant.serialize_entry("kind", "unmatched_expectation")?;
                variant.serialize_entry("index", index)?;
                variant.serialize_entry("expectation", &expectation)?;
                variant.end()
            }
            DiffLine::UnexpectedLines { lines } => {
                let mut variant = serializer.serialize_map(Some(2))?;
                variant.serialize_entry("kind", "unexpected_lines")?;
                variant.serialize_entry("lines", &lines_to_strings(lines))?;
                variant.end()
            }
        }
    }
}

fn lines_to_strings(lines: &[(usize, Vec<u8>)]) -> Vec<(usize, String)> {
    lines
        .iter()
        .map(|(index, line)| (*index, lossy_string!(line)))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::DiffLine;
    use super::DiffTool;
    use crate::bformatln;
    use crate::blines;
    use crate::diff::Diff;
    use crate::test_expectation;

    #[test]
    fn test_exact_match() {
        let differ = DiffTool {
            expectations: vec![test_expectation!("equal", "foo")],
        };

        let diffs = differ.diff(&bformatln!("foo")).expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_exact_no_match() {
        let differ = DiffTool {
            expectations: vec![test_expectation!("equal", "bar")],
        };

        let diffs = differ.diff(&bformatln!("foo")).expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_quantifiers_optional() {
        let tests = &[
            (
                DiffTool {
                    expectations: vec![test_expectation!("equal", "foo", false, false)],
                },
                vec![],
                false,
            ),
            (
                DiffTool {
                    expectations: vec![test_expectation!("equal", "foo", false, false)],
                },
                blines!("foo"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![test_expectation!("equal", "foo", true, false)],
                },
                vec![],
                true,
            ),
            (
                DiffTool {
                    expectations: vec![test_expectation!("equal", "foo", true, false)],
                },
                blines!("foo"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![
                        test_expectation!("equal", "foo", false, false),
                        test_expectation!("equal", "bar", false, false),
                        test_expectation!("equal", "baz", false, false),
                    ],
                },
                blines!("foo", "bar", "baz"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![
                        test_expectation!("equal", "foo", true, false),
                        test_expectation!("equal", "bar", true, false),
                        test_expectation!("equal", "baz", true, false),
                    ],
                },
                blines!("foo", "bar", "baz"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![
                        test_expectation!("equal", "foo", true, false),
                        test_expectation!("equal", "bar", true, false),
                        test_expectation!("equal", "baz", true, false),
                    ],
                },
                blines!("bar", "baz"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![
                        test_expectation!("equal", "foo", true, false),
                        test_expectation!("equal", "bar", true, false),
                        test_expectation!("equal", "baz", true, false),
                    ],
                },
                blines!("foo", "baz"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![
                        test_expectation!("equal", "foo", true, false),
                        test_expectation!("equal", "bar", true, false),
                        test_expectation!("equal", "baz", true, false),
                    ],
                },
                blines!("foo", "bar"),
                true,
            ),
            (
                DiffTool {
                    expectations: vec![
                        test_expectation!("equal", "foo", true, false),
                        test_expectation!("equal", "bar", true, false),
                        test_expectation!("equal", "baz", true, false),
                    ],
                },
                vec![],
                true,
            ),
        ];

        for (idx, (differ, lines, expect)) in tests.iter().enumerate() {
            let diffs = differ.diff(lines).expect("diff created");
            assert_eq!(
                !diffs.has_differences(),
                *expect,
                "test {}: for input empty = {} with optional = {} -> {:?}",
                idx + 1,
                lines.is_empty(),
                differ.expectations[0].optional,
                diffs
            )
        }
    }

    #[test]
    fn test_unmatched_middle_expectation() {
        let differ = make();

        let diffs = differ.diff(&blines!("foo", "baz")).expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unmatched_tailing_expectation() {
        let differ = make();

        let diffs = differ.diff(&blines!("foo", "bar")).expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unmatched_and_unexpected() {
        let differ = make();

        let diffs = differ
            .diff(&blines!("foo", "bar", "zoing"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unmatched_heading_expectation() {
        let differ = make();

        let diffs = differ.diff(&blines!("bar", "baz")).expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unexpected_heading_lines() {
        let differ = make();

        let diffs = differ
            .diff(&blines!("something", "bla", "foo", "bar", "baz"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unexpected_intermediate_lines() {
        let differ = make();

        let diffs = differ
            .diff(&blines!("foo", "bar", "something", "bla", "baz"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unexpected_tailing_lines() {
        let differ = make();

        let diffs = differ
            .diff(&blines!("foo", "bar", "baz", "something", "more"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_unused_expectations() {
        let differ = make();

        let diffs = differ.diff(&blines!("foo")).expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_multiline_expectation() {
        let differ = DiffTool {
            expectations: vec![test_expectation!("glob", "f*", false, true)],
        };

        let diffs = differ
            .diff(&blines!("foo", "fun", "fact"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_starting_multiline_expectation() {
        let differ = DiffTool {
            expectations: vec![
                test_expectation!("glob", "f*", false, true),
                test_expectation!("equal", "bar", false, true),
            ],
        };

        let diffs = differ
            .diff(&blines!("foo", "fun", "fact", "bar"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_middle_multiline_expectation() {
        let differ = DiffTool {
            expectations: vec![
                test_expectation!("equal", "baz", false, true),
                test_expectation!("glob", "f*", false, true),
                test_expectation!("equal", "bar", false, true),
            ],
        };

        let diffs = differ
            .diff(&blines!("baz", "foo", "fun", "fact", "bar"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_tailing_multiline_expectation() {
        let differ = DiffTool {
            expectations: vec![
                test_expectation!("equal", "baz", false, true),
                test_expectation!("glob", "f*", false, true),
            ],
        };

        let diffs = differ
            .diff(&blines!("baz", "foo", "fun", "fact"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    // Tests need to fail if not all (non-optional) expectations are matched.
    // this test assures that this is the also the case when expectations follow
    // a multiline expectation. The following MUST FAIL, because the last
    // expectation does NOT match and is "left over":
    // ```scrut
    // $ echo -e "foo\nfoo\nbar"
    // * (glob*)
    // baz
    // ```
    #[test]
    fn test_regression_excess_expectations_after_multiline_fail() {
        let differ = DiffTool {
            expectations: vec![
                test_expectation!("glob", "*", false, true),
                test_expectation!("equal", "bar", false, true),
            ],
        };

        let diffs = differ
            .diff(&blines!("foo", "foo", "baz"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    // Related to test_regression_excess_expectations_after_multiline_fail, expectations
    // that follow (greedy) multiline expectations, but are not multiline expectations
    // themselves (and thereby "more precise") must tak precedence and "end" the
    // multiline match. The following MUST WORK:
    // ```scrut
    // $ echo -e "foo\nfoo\nbar"
    // * (glob*)
    // bar
    // ```
    #[test]
    fn test_matching_non_multiline_precedent_over_matching_multiline() {
        let differ = DiffTool {
            expectations: vec![
                test_expectation!("glob", "*", false, true),
                test_expectation!("equal", "bar", false, true),
            ],
        };

        let diffs = differ
            .diff(&blines!("foo", "foo", "bar"))
            .expect("no error");
        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_next_expectation_is_used_first() {
        let differ = DiffTool {
            expectations: vec![
                test_expectation!("equal", "foo"),
                test_expectation!("equal", ""),
                test_expectation!("equal", "bar"),
                test_expectation!("equal", ""),
                test_expectation!("equal", "baz"),
                test_expectation!("equal", ""),
                test_expectation!("equal", "zoing"),
            ],
        };

        let diffs = differ
            .diff(&blines!("foo", "", "baz", "", "zoing"))
            .expect("no error");

        insta::assert_debug_snapshot!(diffs);
    }

    #[test]
    fn test_serialize() {
        let diff = Diff::new(vec![
            DiffLine::MatchedExpectation {
                index: 0,
                expectation: test_expectation!("equal", "matched", false, false),
                lines: vec![(0, bformatln!("line content"))],
            },
            DiffLine::UnmatchedExpectation {
                index: 0,
                expectation: test_expectation!("equal", "unmatched", false, false),
            },
            DiffLine::UnexpectedLines {
                lines: vec![(0, bformatln!("line content"))],
            },
        ]);
        let rendered = serde_yaml::to_string(&diff).expect("render to yaml");
        insta::assert_snapshot!(&rendered);
    }

    fn make() -> DiffTool {
        DiffTool {
            expectations: vec![
                test_expectation!("equal", "foo"),
                test_expectation!("equal", "bar"),
                test_expectation!("equal", "baz"),
            ],
        }
    }
}
