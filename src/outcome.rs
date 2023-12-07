/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use serde::ser::SerializeMap;
use serde::Serialize;

use crate::escaping::Escaper;
use crate::output::Output;
use crate::parsers::parser::ParserType;
use crate::testcase::Result as TestCaseResult;
use crate::testcase::TestCase;

/// Aggregation of all that a renderer could possibly need to build a readable,
/// understandable output
pub struct Outcome {
    /// The path / URL of the test
    pub location: Option<String>,

    /// The output that this outcome describes
    pub output: Output,

    /// The testcase that this outcome is about
    pub testcase: TestCase,

    /// The original format the testcase was written in
    pub format: ParserType,

    // The output escaping mode this outcome needs to have
    pub escaping: Escaper,

    /// The result of validating the [`crate::testcase::TestCase`]
    pub result: TestCaseResult<()>,
}

impl Serialize for Outcome {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut count = 2;
        if self.location.is_some() {
            count += 1;
        }
        if self.result.is_err() {
            count += 1;
        }
        let mut outcome = serializer.serialize_map(Some(count))?;
        if let Some(ref location) = self.location {
            outcome.serialize_entry("location", location)?;
        }
        match &self.result {
            Err(err) => {
                outcome.serialize_entry("output", &self.output)?;
                outcome.serialize_entry("testcase", &self.testcase)?;
                outcome.serialize_entry("result", err)?;
            }
            Ok(_) => {
                outcome.serialize_entry("title", &self.testcase.title)?;
                let mut map = HashMap::new();
                map.insert("kind", "success");
                outcome.serialize_entry("result", &map)?;
            }
        }
        outcome.end()
    }
}

#[cfg(test)]
mod tests {
    use super::Outcome;
    use crate::escaping::Escaper;
    use crate::parsers::parser::ParserType;
    use crate::testcase::TestCaseError;

    #[test]
    fn test_serialize() {
        use crate::test_expectation;
        use crate::testcase::TestCase;

        let outcomes = vec![
            (
                "error",
                Outcome {
                    location: Some("path/file.md".to_string()),
                    output: ("stdout", "stderr", Some(123)).into(),
                    testcase: TestCase {
                        title: "the title".to_string(),
                        shell_expression: "the command".to_string(),
                        expectations: vec![test_expectation!("equal", "foo")],
                        exit_code: Some(234),
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Err(TestCaseError::InvalidExitCode {
                        actual: 123,
                        expected: 234,
                    }),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
            (
                "success",
                Outcome {
                    location: Some("path/file.md".to_string()),
                    output: ("stdout", "stderr", Some(123)).into(),
                    testcase: TestCase {
                        title: "the title".to_string(),
                        shell_expression: "the command".to_string(),
                        expectations: vec![test_expectation!("equal", "foo")],
                        exit_code: Some(123),
                        line_number: 234,
                        ..Default::default()
                    },
                    result: Ok(()),
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                },
            ),
        ];

        for (name, outcome) in outcomes {
            insta::assert_json_snapshot!(name, outcome);
        }
    }
}
