/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Result;
use colored::ColoredString;
use colored::Colorize;

use crate::formatln;
use crate::outcome::Outcome;

const MAX_LINE_LENGTH: usize = 80;

pub(super) trait OutcomeHeader {
    fn render_header(&self) -> Result<String>;
}

impl OutcomeHeader for Outcome {
    fn render_header(&self) -> Result<String> {
        let mut headers = vec![];
        if let Some(ref location) = self.location {
            headers.push(header_to_title(
                "@",
                &format!("{}:{}", location, self.testcase.line_number),
                |s| s.bright_blue(),
            ));
        } else {
            headers.push(header_to_title(
                "@",
                &format!("Line {}", self.testcase.line_number),
                |s| s.bright_blue(),
            ));
        }
        if !self.testcase.title.is_empty() {
            headers.push(header_to_title("#", &self.testcase.title, |s| {
                s.bright_cyan()
            }));
        }
        headers.push(header_to_title("$", &self.testcase.shell_expression, |s| {
            s.bold().bright_yellow()
        }));

        let divider_outer = &formatln!("// {}", "=".repeat(MAX_LINE_LENGTH - 3))
            .bright_black()
            .to_string();
        let divider_inner = &formatln!("// {}", "-".repeat(MAX_LINE_LENGTH - 3))
            .bright_black()
            .to_string();

        let mut output = String::new();
        output.push_str(divider_outer);
        output.push_str(&headers.join(divider_inner));
        output.push_str(divider_outer);
        output.push('\n');

        Ok(output)
    }
}

fn header_to_title(first_prefix: &str, text: &str, color: fn(&str) -> ColoredString) -> String {
    let mut title = String::new();
    let prefix = "//".bright_black();
    for (index, line) in text.split('\n').enumerate() {
        title.push_str(&formatln!(
            "{} {} {}",
            prefix,
            (if index == 0 { first_prefix } else { " " }).bold(),
            color(line)
        ))
    }
    title
}
