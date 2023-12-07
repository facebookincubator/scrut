/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Result;

use crate::outcome::Outcome;

/// Takes whole test documents, in the appropriate syntax of the implementation,
/// and returns an updated document, for which all testcases blocks contain
/// updated expectations, that match the current output.
pub trait UpdateGenerator {
    fn generate_update(&self, original_document: &str, outcomes: &[&Outcome]) -> Result<String>;
}

/// Creates parsable text rendering of [`crate::testcase::TestCase`]s in the
/// appropriate syntax of the implementation.
pub trait TestCaseGenerator {
    fn generate_testcases(&self, outcomes: &[&Outcome]) -> Result<String>;
}

#[cfg(test)]
pub(super) mod tests {
    use super::TestCaseGenerator;
    use super::UpdateGenerator;
    use crate::diff::Diff;
    use crate::diff::DiffLine;
    use crate::escaping::Escaper;
    use crate::formatln;
    use crate::outcome::Outcome;
    use crate::parsers::parser::ParserType;
    use crate::testcase::TestCase;
    use crate::testcase::TestCaseError;

    pub(crate) struct UpdateGeneratorTest<'a> {
        pub original_document: &'a str,
        pub outcomes: Vec<Outcome>,
        /* pub testcases: Vec<TestCase>,
        pub outputs: Vec<Output>, */
    }

    pub(crate) fn run_update_generator_tests<T: UpdateGenerator>(
        generator: T,
        name: &str,
        tests: &[(&str, UpdateGeneratorTest)],
    ) {
        for (title, test) in tests {
            let result = generator
                .generate_update(
                    test.original_document,
                    &test.outcomes.iter().collect::<Vec<_>>(),
                    /*  &test.testcases.iter().collect::<Vec<_>>(),
                    &test.outputs.iter().collect::<Vec<_>>(), */
                )
                .expect(title);
            insta::assert_snapshot!(format!("update__{}__{}", name, *title), result);
        }
    }

    pub(crate) fn standard_testcase_generator_test_suite<T: TestCaseGenerator>(
        generator: T,
        name: &str,
    ) {
        let tests: &[(&str, Outcome)] = &[
            (
                "equal_output",
                Outcome {
                    location: None,
                    output: ("the output\n", "").into(),
                    testcase: TestCase {
                        title: "This is a test".to_string(),
                        shell_expression: "the command".to_string(),
                        expectations: vec![],
                        exit_code: None,
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                        DiffLine::UnexpectedLines {
                            lines: vec![(0, formatln!("the output").as_bytes().to_vec())],
                        },
                    ]))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
            (
                "no-eol_output",
                Outcome {
                    location: None,
                    output: ("the output", "").into(),
                    testcase: TestCase {
                        title: "This is a test".to_string(),
                        shell_expression: "the command".to_string(),
                        expectations: vec![],
                        exit_code: None,
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                        DiffLine::UnexpectedLines {
                            lines: vec![(0, b"the output".to_vec())],
                        },
                    ]))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
            (
                "multiline_output",
                Outcome {
                    location: None,
                    output: ("line 1\nline 2\n line3", "").into(),
                    testcase: TestCase {
                        title: "This is a test".to_string(),
                        shell_expression: "the command".to_string(),
                        expectations: vec![],
                        exit_code: None,
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                        DiffLine::UnexpectedLines {
                            lines: vec![
                                (0, formatln!("line 1").as_bytes().to_vec()),
                                (1, formatln!("line 2").as_bytes().to_vec()),
                                (2, b"line 3".to_vec()),
                            ],
                        },
                    ]))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
            (
                "multiline_command",
                Outcome {
                    location: None,
                    output: ("the output\n", "").into(),
                    testcase: TestCase {
                        title: "This is a test".to_string(),
                        shell_expression: "echo \\\nsomething".into(),
                        expectations: vec![],
                        exit_code: None,
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                        DiffLine::UnexpectedLines {
                            lines: vec![(0, formatln!("the output").as_bytes().to_vec())],
                        },
                    ]))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
            (
                "non-zero_exit_code",
                Outcome {
                    location: None,
                    output: ("the output\n", "", Some(123)).into(),
                    testcase: TestCase {
                        title: "This is a test".to_string(),
                        shell_expression: "the command".into(),
                        expectations: vec![],
                        exit_code: Some(123),
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                        DiffLine::UnexpectedLines {
                            lines: vec![(0, formatln!("the output").as_bytes().to_vec())],
                        },
                    ]))),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
        ];

        run_testcase_generator_tests(generator, name, tests);
    }

    pub(crate) fn run_testcase_generator_tests<T: TestCaseGenerator>(
        generator: T,
        name: &str,
        tests: &[(&str, Outcome)],
    ) {
        for (title, outcome) in tests {
            let result = generator
                .generate_testcases(&[outcome])
                .unwrap_or_else(|_| panic!("test {}", *title));
            insta::assert_snapshot!(format!("create__{}__{}", name, *title), result);
        }
    }
}
