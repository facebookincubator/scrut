/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Result;
use serde::Serialize;

use super::escaped_filter::apply_escaped_filter_bytes;
use super::rule::Rule;
use super::rule::RuleMaker;
use crate::newline::BytesNewline;

/// Simple equality match for lines that end in a new-line character
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EscapedRule(String, Vec<u8>);

impl Rule for EscapedRule {
    fn kind(&self) -> &'static str {
        "escaped"
    }

    fn matches(&self, line: &[u8]) -> bool {
        &self.1[..] == line.trim_newlines()
    }

    fn unmake(&self) -> (String, Vec<u8>) {
        (self.kind().to_string(), self.1.to_owned())
    }
}

impl RuleMaker for EscapedRule {
    fn make(expression: &str) -> Result<Box<dyn Rule>> {
        // Cram-Compat: ignore tailing `(no-eol)`, because comparison is without
        // tailing newline anyway.
        let expression = if expression.ends_with(" (no-eol)") {
            &expression[0..(expression.len() - " (no-eol)".len())]
        } else {
            expression
        };

        let bytes = apply_escaped_filter_bytes(expression)?;
        Ok(Box::new(EscapedRule(expression.to_string(), bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::EscapedRule;
    use crate::lossy_string;
    use crate::newline::StringNewline;
    use crate::rules::rule::RuleMaker;

    #[test]
    fn test_make_unmake() {
        let rule = EscapedRule::make("foo").expect("rule is created");
        let (kind, expression) = rule.unmake();
        assert_eq!("escaped", kind);
        assert_eq!("foo", lossy_string!(&expression));
    }

    #[test]
    fn test_rule_matches() {
        let tests = vec![
            (true, "foo", b"foo".to_vec()),
            (true, "foo", "foo".assure_newline().as_bytes().to_vec()),
            (true, "foo\\tbar", b"foo\tbar".to_vec()),
            (false, "foo\\tbar", b"foo\\tbar".to_vec()),
            (true, "foo\\\\nbar", b"foo\\nbar".to_vec()),
            (
                true,
                "\\xe2\\x80\\x9cfoo\\xe2\\x80\\x9d",
                "“foo”".as_bytes().to_vec(),
            ),
            (true, "foo\\xe2\\x80\\x99", "foo’".as_bytes().to_vec()),
            (true, "foo\\tbar", b"foo\tbar".to_vec()),
            (true, "foo\\x00\\x01bar", b"foo\x00\x01bar".to_vec()),
            (true, "foo\\000\\001bar", b"foo\x00\x01bar".to_vec()),
            (
                true,
                "foo \\x1b\\x5b\\x31\\x6dbar",
                b"foo \x1b\x5b\x31\x6dbar".to_vec(),
            ),
            (true, "\\xef\\xbb\\xbf", b"\xef\xbb\xbf".to_vec()),
            (
                true,
                "[\"/* ABC */\\\\n\",\"\\xef\\xbb\\xbf\",[],1]",
                b"[\"/* ABC */\\n\",\"\xef\xbb\xbf\",[],1]\n".to_vec(),
            ),
            (true, "😁", "😁".as_bytes().to_vec()),
            (
                true,
                "\x1b[1m😁\x1b[0m",
                "\x1b[1m😁\x1b[0m".as_bytes().to_vec(),
            ),
        ];

        tests
            .iter()
            .enumerate()
            .for_each(|(i, (expect, expression, line))| {
                let rule = EscapedRule::make(expression)
                    .unwrap_or_else(|_| panic!("create rule from {expression}"));
                assert_eq!(
                    *expect,
                    rule.matches(line),
                    "{:02} expression = `{}` with expected = {} matches {:?}",
                    i + 1,
                    expression,
                    *expect,
                    lossy_string!(line),
                )
            });
    }

    #[test]
    fn test_rule_serialize() {
        let rule = EscapedRule::make("abc").unwrap();
        let serialized = serde_json::to_string(&rule).expect("serialize");
        assert_eq!("{\"kind\":\"escaped\",\"expression\":\"abc\"}", serialized);
    }
}
