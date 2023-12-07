/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! The glob rule implementation for Scrut uses the [`wildmatch`] crate which
//! supports simple matches against wildcards `*` and `?`
//!
//! Alternative crates that have been reviewed:
//! - [`globset`] - does not support non-printable characters (eg color escape
//!   sequences like `\x1b[1mfoo`) and does handle `/` in an incompatible way
//!   when comparing against arbitrary text (eg `foo / bar / baz` is not
//!   matching `*bar*`)
//! - [``]
//!
use std::fmt::Display;

use anyhow::Result;
use wildmatch::WildMatch;

use super::escaped_filter::apply_escaped_filter_utf8;
use super::escaped_filter::expression_as_escaped;
use super::rule::Rule;
use super::rule::RuleMaker;
use crate::lossy_string;
use crate::newline::BytesNewline;

/// Simple equality match for lines that end in a new-line character
#[derive(Clone, Debug, PartialEq)]
pub struct GlobRule(WildMatch);

impl Display for GlobRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Rule for GlobRule {
    fn kind(&self) -> &'static str {
        "glob"
    }

    fn matches(&self, line: &[u8]) -> bool {
        self.0.matches(&lossy_string!(line.trim_newlines()))
    }

    fn unmake(&self) -> (String, Vec<u8>) {
        (
            self.kind().to_string(),
            self.0.to_string().as_bytes().to_vec(),
        )
    }
}

impl RuleMaker for GlobRule {
    fn make(expression: &str) -> Result<Box<dyn Rule>> {
        let expression = if let Some(expression) = expression_as_escaped(expression) {
            apply_escaped_filter_utf8(expression)?
        } else {
            expression.to_string()
        };
        Ok(Box::new(Self(WildMatch::new(&expression))))
    }
}

#[cfg(test)]
mod tests {
    use super::GlobRule;
    use crate::lossy_string;
    use crate::newline::StringNewline;
    use crate::rules::rule::RuleMaker;

    #[test]
    fn test_make_unmake() {
        let tests = vec![
            ("foo", "foo"),
            ("fo\\o", "fo\\o"),
            ("foo \x1b[1mbar\x1b[0m", "foo \u{1b}[1mbar\u{1b}[0m"),
            ("fo\\o \x1b[1mbar\x1b[0m", "fo\\o \u{1b}[1mbar\u{1b}[0m"),
        ];

        for (from, expect) in tests {
            let rule =
                GlobRule::make(from).unwrap_or_else(|_| panic!("rule is created from `{}`", from));
            let (kind, expression) = rule.unmake();
            assert_eq!("glob", kind);
            assert_eq!(expect, lossy_string!(&expression));
        }
    }

    #[test]
    fn test_rule_matches() {
        let tests = vec![
            (true, "foo", "foo".to_string()),
            (true, "foo", "foo".assure_newline().to_string()),
            (true, "*foo", "ends in foo".to_string()),
            (true, "*foo*", "barfoobaz".to_string()),
            (true, "*foo*", "somewhere foo word".to_string()),
            (false, "*bar*", "somewhere foo word".to_string()),
            (true, "foo*", "foo at start".to_string()),
            (false, "foo*", "not starting with foo".to_string()),
            (false, "foo?", "foo".to_string()),
            (true, "foo?", "foop".to_string()),
            (true, "foo??", "foopp".to_string()),
            (true, "foo??*", "foobar".to_string()),
            (true, "foo\\x1d[1mbar* (esc)", "foo\x1d[1mbarbaz".to_string()),
            (
                true,
                r#""description": "`*foo* with code 1\n--- stdout\nthis is stdout\n--- stderr\nthis is stderr""#,
                r#""description": "`/tmp/execution.123123/bla/test/foo --bar --baz with code 1\n--- stdout\nthis is stdout\n--- stderr\nthis is stderr""#.to_string(),
            ),
        ];

        tests.iter().for_each(|(expect, expression, line)| {
            let rule = GlobRule::make(expression)
                .unwrap_or_else(|_| panic!("create rule from {}", expression));
            assert_eq!(*expect, rule.matches(line.as_bytes()),)
        });
    }

    #[test]
    fn test_rule_serialize() {
        let rule = GlobRule::make("abc").unwrap();
        let serialized = serde_json::to_string(&rule).expect("serialize");
        assert_eq!("{\"kind\":\"glob\",\"expression\":\"abc\"}", serialized);
    }
}
