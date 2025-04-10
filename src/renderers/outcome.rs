/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Result;
use console::style;
use console::StyledObject;

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
                |s| style(s).bright().blue(),
            ));
        } else {
            headers.push(header_to_title(
                "@",
                &format!("Line {}", self.testcase.line_number),
                |s| style(s).bright().blue(),
            ));
        }
        if !self.testcase.title.is_empty() {
            headers.push(header_to_title("#", &self.testcase.title, |s| {
                style(s).bright().cyan()
            }));
        }
        headers.push(header_to_title("$", &self.testcase.shell_expression, |s| {
            style(s).bold().bright().yellow()
        }));

        let divider_outer = style(formatln!("// {}", "=".repeat(MAX_LINE_LENGTH - 3)))
            .bright()
            .black()
            .to_string();
        let divider_inner = &style(formatln!("// {}", "-".repeat(MAX_LINE_LENGTH - 3)))
            .bright()
            .black()
            .to_string();

        let mut output = String::new();
        output.push_str(&divider_outer);
        output.push_str(&headers.join(divider_inner));
        output.push_str(&divider_outer);
        output.push('\n');

        Ok(output)
    }
}

fn header_to_title(
    first_prefix: &str,
    text: &str,
    color: fn(&str) -> StyledObject<&str>,
) -> String {
    let mut title = String::new();
    let prefix = style("//").bright().black();
    for (index, line) in text.split('\n').enumerate() {
        title.push_str(&formatln!(
            "{} {} {}",
            prefix,
            style(if index == 0 { first_prefix } else { " " }).bold(),
            color(line)
        ))
    }
    title
}
