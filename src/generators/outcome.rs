use anyhow::bail;
use anyhow::Result;

use crate::diff::DiffLine;
use crate::formatln;
use crate::lossy_string;
use crate::newline::BytesNewline;
use crate::newline::SplitLinesByNewline;
use crate::newline::StringNewline;
use crate::outcome::Outcome;
use crate::output::ExitStatus;
use crate::testcase::TestCaseError;

pub(super) trait OutcomeTestGenerator {
    fn generate_testcase(&self) -> Result<String>;
}

impl Outcome {
    fn generate_testcase_expression(&self) -> String {
        // prepend by command
        let expression_lines = self.testcase.shell_expression.as_bytes();
        let expression_lines = expression_lines.split_at_newline();
        let mut generated = format!("$ {}", lossy_string!(&expression_lines[0].assure_newline()));
        expression_lines.iter().skip(1).for_each(|line| {
            generated.push_str(&format!(
                "> {}",
                lossy_string!(&(&line[..]).assure_newline())
            ))
        });
        generated
    }

    fn generate_testcase_exit_code(&self) -> Option<String> {
        match &self.output.exit_code {
            ExitStatus::Code(code) if *code != 0 => Some(formatln!("[{}]", code)),
            _ => None,
        }
    }
}

impl OutcomeTestGenerator for Outcome {
    fn generate_testcase(&self) -> Result<String> {
        match &self.result {
            Ok(_) => {
                let mut generated = self.generate_testcase_expression();
                self.testcase.expectations.iter().for_each(|expectation| {
                    generated.push_str(&expectation.original_string().assure_newline())
                });
                if let Some(exit_code) = self.generate_testcase_exit_code() {
                    generated.push_str(&exit_code)
                }
                Ok(generated)
            }
            Err(err) => match err {
                TestCaseError::MalformedOutput(diff) => {
                    let mut generated = self.generate_testcase_expression();

                    // output the actual recorded output lines
                    for diff_line in diff.lines.iter() {
                        match diff_line {
                            DiffLine::MatchedExpectation {
                                index: _,
                                expectation,
                                lines: _,
                            } => {
                                generated.push_str(&expectation.original_string().assure_newline())
                            }
                            DiffLine::UnexpectedLines { lines } => {
                                for (_, line) in lines {
                                    let suffix = if line.ends_with(b"\n") {
                                        ""
                                    } else {
                                        " (no-eol)"
                                    };
                                    let line = formatln!(
                                        "{}{}",
                                        self.escaping
                                            .escaped_expectation((&line[..]).trim_newlines()),
                                        suffix
                                    );
                                    generated.push_str(&line)
                                }
                            }
                            _ => continue,
                        }
                    }
                    if let Some(exit_code) = self.generate_testcase_exit_code() {
                        generated.push_str(&exit_code)
                    }
                    Ok(generated)
                }
                TestCaseError::InvalidExitCode {
                    actual,
                    expected: _,
                } => {
                    let mut generated = self.generate_testcase_expression();
                    let mut output = self.output.stdout.to_output_string(None, &self.escaping);
                    if !output.is_empty() && !output.ends_with('\n') {
                        output.push_str(" (no-eol)\n")
                    }
                    generated.push_str(&output);
                    generated.push_str(&formatln!("[{}]", *actual));
                    Ok(generated)
                }
                TestCaseError::InternalError(err) => {
                    bail!("cannot generate testcase from internal error: {}", err)
                }
                TestCaseError::Skipped => {
                    bail!("cannot generate skipped testcase")
                }
            },
        }
    }
}
