use anyhow::Result;

use super::generator::TestCaseGenerator;
use super::generator::UpdateGenerator;
use super::outcome::OutcomeTestGenerator;
use crate::formatln;
use crate::outcome::Outcome;
use crate::parsers::cram::DEFAULT_CRAM_INDENTION;

/// Update [`crate::testcase::TestCase`]s in an existing Cram document
pub struct CramUpdateGenerator {
    pub indention: usize,
}

impl CramUpdateGenerator {
    pub fn new(indention: usize) -> Self {
        Self { indention }
    }
}

impl Default for CramUpdateGenerator {
    fn default() -> Self {
        Self::new(DEFAULT_CRAM_INDENTION)
    }
}

impl UpdateGenerator for CramUpdateGenerator {
    fn generate_update(
        &self,
        original_document: &str,
        outcomes: &[&Outcome],
        /* testcase: &[&TestCase],
        outputs: &[&Output], */
    ) -> Result<String> {
        if outcomes.is_empty() {
            return Ok(original_document.into());
        }

        let indent = " ".repeat(self.indention);
        let mut testcases = vec![];
        for outcome in outcomes {
            let mut testcase = if outcome.testcase.title.is_empty() {
                "".into()
            } else {
                formatln!("{}", outcome.testcase.title)
            };
            testcase.push_str(&cram_indented(&indent, &outcome.generate_testcase()?));
            testcases.push(testcase);
        }

        Ok(testcases.join("\n\n"))
    }
}

/// Generate a new Cram [`crate::testcase::TestCase`] document from shell
/// expression and it's [`crate::output::Output`]
pub struct CramTestCaseGenerator {
    pub indention: usize,
}

impl CramTestCaseGenerator {
    pub fn new(indention: usize) -> Self {
        Self { indention }
    }
}

impl Default for CramTestCaseGenerator {
    fn default() -> Self {
        Self::new(DEFAULT_CRAM_INDENTION)
    }
}

impl TestCaseGenerator for CramTestCaseGenerator {
    fn generate_testcases(&self, outcomes: &[&Outcome]) -> Result<String> {
        outcomes
            .iter()
            .map(|outcome| {
                let mut rendered = String::new();
                if !outcome.testcase.title.is_empty() {
                    rendered.push_str(&outcome.testcase.title);
                    rendered.push('\n');
                }

                let indent = " ".repeat(self.indention);
                let generated = outcome.generate_testcase()?;
                rendered.push_str(&cram_indented(&indent, &generated));

                Ok(rendered)
            })
            .collect::<Result<Vec<_>>>()
            .map(|result| result.join("\n\n"))
    }
}

fn cram_indented(indent: &str, from: &str) -> String {
    if from.is_empty() {
        "".into()
    } else {
        from.trim_end()
            .split('\n')
            .map(|line| format!("{}{}", indent, line))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::CramTestCaseGenerator;
    use super::CramUpdateGenerator;
    use crate::diff::Diff;
    use crate::diff::DiffLine;
    use crate::escaping::Escaper;
    use crate::formatln;
    use crate::generators::generator::tests::run_update_generator_tests;
    use crate::generators::generator::tests::standard_testcase_generator_test_suite;
    use crate::generators::generator::tests::UpdateGeneratorTest;
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
                    original_document: "This is a test\n  $ the command\n  an expectation\n",
                    outcomes: vec![Outcome {
                        location: None,
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
                        },
                        output: ("an expectation\n", "").into(),
                        result: Ok(()),
                        escaping: Escaper::default(),
                        format: ParserType::Cram,
                    }],
                },
            ),
            (
                "complex_unchanged",
                UpdateGeneratorTest {
                    original_document: "This is a test\n  $ the command\n  line * (glob+)\n",
                    outcomes: vec![Outcome {
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
                        },
                        output: ("line 1\nline 2\nline 3\n", "").into(),
                        result: Ok(()),
                        location: None,
                        escaping: Escaper::default(),
                        format: ParserType::Cram,
                    }],
                },
            ),
            (
                "updated_output",
                UpdateGeneratorTest {
                    original_document: "This is a test\n  $ the command\n  an expectation\n",
                    outcomes: vec![Outcome {
                        testcase: TestCase {
                            title: "This is a test".to_string(),
                            shell_expression: "the command".to_string(),
                            expectations: vec![test_expectation!("equal", "an expectation")],
                            exit_code: None,
                            line_number: 234,
                        },
                        output: ("new output\n", "").into(),
                        result: Err(TestCaseError::MalformedOutput(Diff::new(vec![
                            DiffLine::UnmatchedExpectation {
                                index: 0,
                                expectation: test_expectation!("equal", "an expectation"),
                            },
                            DiffLine::UnexpectedLines {
                                lines: vec![(0, formatln!("new output").as_bytes().to_vec())],
                            },
                        ]))),
                        location: None,
                        escaping: Escaper::default(),
                        format: ParserType::Cram,
                    }],
                },
            ),
        ];

        let generator = CramUpdateGenerator::default();
        run_update_generator_tests(generator, "cram", tests);
    }

    #[test]
    fn test_testcase_generator() {
        let generator = CramTestCaseGenerator::default();
        standard_testcase_generator_test_suite(generator, "cram");
    }
}
