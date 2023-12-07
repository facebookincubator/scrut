/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Display;

use anyhow::Result;
use regex::bytes::Regex as ByteRegex;
use regex::Captures;
use regex::Regex;

use super::rule::Rule;
use super::rule::RuleMaker;
use crate::newline::BytesNewline;

/// Simple equality match for lines that end in a new-line character
#[derive(Clone, Debug)]
pub struct RegexRule(String, ByteRegex);

impl Display for RegexRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for RegexRule {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Rule for RegexRule {
    fn kind(&self) -> &'static str {
        "regex"
    }

    fn matches(&self, line: &[u8]) -> bool {
        self.1.is_match(line.trim_newlines())
    }

    fn unmake(&self) -> (String, Vec<u8>) {
        (self.kind().to_string(), self.0.as_bytes().to_vec())
    }
}

impl RuleMaker for RegexRule {
    fn make(expression: &str) -> Result<Box<dyn Rule>> {
        let expression = cleanup_unrecognized_escape_sequences(expression);
        let expression = escape_misused_repetition_quantifier(&expression);
        let expression = escape_misused_character_class(&expression);
        let regex = ByteRegex::new(&format!("^{}$", expression))?;
        Ok(Box::new(RegexRule(expression, regex)))
    }
}

lazy_static! {
    static ref VALID_REPETITION_QUANTIFIER: Regex = Regex::new("\\{([0-9]+(?:,[0-9]+)?)\\}")
        .expect("valid repetition quantifier regex must compile");
    static ref MOVED_REPETITION_QUANTIFIER: Regex =
        Regex::new("<<<<(.+?)>>>>").expect("moved repetition quantifier regex must compile");
}

/// Compensate for misuse of repetition quantifiers, where curly brackets are used
/// as regular strings:
/// 1. normal use of quantifiers: `something{3,6}` < denotes that the last `g` occurs 3-6 times
/// 2. escaped use in strings: `hello\{world\}`
/// 3. misuse, unescaped in strings: `hello{world}`
/// The `regex` crate implementation of regular expressions is strict and would
/// not allow for case (3) and throw an error.
/// The following implementation is a best effort conversion from (3) to (2).
///
/// Reasons:
/// - Python does it so, existing cram tests may have it
/// - Usability (?)
pub(super) fn escape_misused_repetition_quantifier(expression: &str) -> String {
    // pass 1: replace all valid repetition quantifiers temporarily
    let expression = VALID_REPETITION_QUANTIFIER.replace_all(expression, |captures: &Captures| {
        format!("<<<<{}>>>>", captures.get(1).unwrap().as_str())
    });

    // pass 2: escape all other curly expressions
    let mut chars = expression.chars();
    let mut expression = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                expression.push(ch);
                if let Some(ch2) = chars.next() {
                    expression.push(ch2);
                }
            }
            '{' | '}' => {
                expression.push('\\');
                expression.push(ch);
            }
            _ => expression.push(ch),
        }
    }

    // pass 3: restore valid repetitions
    let expression = MOVED_REPETITION_QUANTIFIER.replace_all(&expression, |captures: &Captures| {
        ["{", captures.get(1).unwrap().as_str(), "}"].join("")
    });
    expression.to_string()
}

/// Compensate for misuse of character classes, where unescaped square brackets
/// are used within character classes:
/// 1. normal use: `[a-z]` or `[abc]`
/// 2. normal use with escaped special characters: `[\[\]]`
/// 3. misuse, lacking escapes of special characters: `[[]]`
/// The following implementation is a best effort conversion from (3) to (2).
///
/// Reasons:
/// - Python does it so, existing cram tests may have it
/// - Usability (?)
pub(super) fn escape_misused_character_class(expression: &str) -> String {
    let chars = expression.chars().collect::<Vec<_>>();
    let mut index = 0;
    let mut expression = String::new();
    let mut in_cc = false;
    let actual_closing_index = |idx: usize| {
        let mut idx = idx;
        while idx < chars.len() {
            if chars[idx - 1] != '\\' {
                if chars[idx] == ']' {
                    return Some(idx);
                } else if chars[idx] == '[' {
                    return None;
                }
            }
            idx += 1;
        }
        None
    };
    while index < chars.len() {
        let ch = chars[index];
        index += 1;
        match ch {
            '\\' => {
                expression.push(ch);
                if let Some(ch2) = chars.get(index) {
                    expression.push(*ch2);
                    index += 1;
                }
            }
            '[' => {
                if in_cc {
                    expression.push('\\');
                } else {
                    in_cc = true;
                }
                expression.push(ch);
            }
            ']' => {
                if in_cc && actual_closing_index(index).is_none() {
                    in_cc = false;
                } else {
                    expression.push('\\');
                }
                expression.push(ch);
            }
            _ => expression.push(ch),
        }
    }
    expression
}

/// Compensate for use of unnecessary / misused escape:
/// 1. required escape: `foo \[\]`
/// 2. not required escape: `foo\_bar`
/// The following implementation is a best effort removing all unnecessary escapes
///
/// Reasons:
/// - Python does it so, existing cram tests may have it
/// - Usability (?)
pub(super) fn cleanup_unrecognized_escape_sequences(expression: &str) -> String {
    let mut chars = expression.chars();
    let mut expression = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(ch2) = chars.next() {
                    match ch2 {
                        '['
                        | ']'
                        | '{'
                        | '}'
                        | '('
                        | ')'
                        | '|'
                        | '?'
                        | '*'
                        | '+'
                        | '-'
                        | '.'
                        | '^'
                        | '$'
                        | '\\'
                        | ('a'..='z')
                        | ('A'..='Z') => {
                            expression.push(ch);
                        }
                        _ => {}
                    }
                    expression.push(ch2);
                } else {
                    expression.push(ch);
                }
            }
            _ => expression.push(ch),
        }
    }
    expression
}

#[cfg(test)]
mod tests {
    use super::cleanup_unrecognized_escape_sequences;
    use super::escape_misused_character_class;
    use super::escape_misused_repetition_quantifier;
    use super::RegexRule;
    use crate::lossy_string;
    use crate::newline::StringNewline;
    use crate::rules::rule::RuleMaker;

    #[test]
    fn test_make_unmake() {
        let rule = RegexRule::make("foo").expect("rule is created");
        let (kind, expression) = rule.unmake();
        assert_eq!("regex", kind);
        assert_eq!("foo", lossy_string!(&expression));
    }

    #[test]
    fn test_rule_matches() {
        let tests = vec![
            (true, "foo", "foo".to_string()),
            (true, "foo", "foo".assure_newline().to_string()),
            (true, ".*foo.*", "somewhere there is foo word".to_string()),
            (
                true,
                ".*foo.*",
                "also bazfoobar can be one word".to_string(),
            ),
            (true, "foo.*", "foo must be at the start".to_string()),
            (false, "foo.*", "not if foo is not at start".to_string()),
            (
                false,
                "foo.*",
                "not if foo is not at start".assure_newline().to_string(),
            ),
            (true, ".*foo", "things that end in foo".to_string()),
            (
                true,
                ".*foo",
                "things that end in foo".assure_newline().to_string(),
            ),
            (false, ".*foo", "foo is not at the end".to_string()),
            (
                false,
                ".*foo",
                "foo is not at the end".assure_newline().to_string(),
            ),
            (true, "foO{3}", "foOOO".to_string()),
            (true, "foO{3,5}", "foOOOOO".to_string()),
            (false, "foO{3}", "foOOOOO".to_string()),
            (true, "foO{abc}", "foO{abc}".to_string()),
            (true, "foO\\{bcd\\}bar", "foO{bcd}bar".to_string()),
            (true, "{abc}", "{abc}".to_string()),
            (true, "\\{bcd\\}", "{bcd}".to_string()),
            (true, "{cd{ef}}", "{cd{ef}}".to_string()),
            (true, "\\{cd\\{ef\\}\\}", "{cd{ef}}".to_string()),
            (true, "f[oa]o", "fao".to_string()),
            (true, "f[oa]o", "foo".to_string()),
            (true, "f[[]]o", "f[o".to_string()),
            (true, "f[[]]o", "f]o".to_string()),
            (true, "f[x[]o", "f[o".to_string()),
            (true, "f[x[]o", "fxo".to_string()),
            (
                true,
                "foo/\\S+:\\d+:\\d+.*something.*",
                "foo/bar/baz:123:1\tand then something".to_string(),
            ),
        ];

        tests.iter().for_each(|(expect, expression, line)| {
            let rule = RegexRule::make(expression)
                .unwrap_or_else(|_| panic!("create rule from {}", expression));
            assert_eq!(*expect, rule.matches(line.as_bytes()),)
        });
    }

    #[test]
    fn test_rule_serialize() {
        let rule = RegexRule::make("abc").unwrap();
        let serialized = serde_json::to_string(&rule).expect("serialize");
        assert_eq!("{\"kind\":\"regex\",\"expression\":\"abc\"}", serialized);
    }

    #[test]
    fn test_escape_misused_repetition_quantifier() {
        let tests = vec![
            ("foo", "foo"),
            ("foo{bar}", "foo\\{bar\\}"),
            ("foo{123}", "foo{123}"),
            ("foo{12,34}", "foo{12,34}"),
            ("foo{12{34}", "foo\\{12{34}"),
            ("{foo}", "\\{foo\\}"),
        ];

        for (from, expect) in tests {
            let to = escape_misused_repetition_quantifier(from);
            assert_eq!(expect, &to, "from `{}`", from);
        }
    }

    #[test]
    fn test_escape_misused_character_class() {
        let tests = vec![
            ("foo", "foo"),
            ("foo[bar]", "foo[bar]"),
            ("foo[ba[r]", "foo[ba\\[r]"),
            ("foo[[]]", "foo[\\[\\]]"),
            ("foo[[[]]]", "foo[\\[\\[\\]\\]]"),
            (
                "I[0-9]{4} [0-9][0-9]:[0-9][0-9]:[0-9][0-9]\\.[0-9]{6}\\s+\\d+ \\[main\\] abc\\.rs:\\d+\\] something (re)",
                "I[0-9]{4} [0-9][0-9]:[0-9][0-9]:[0-9][0-9]\\.[0-9]{6}\\s+\\d+ \\[main\\] abc\\.rs:\\d+\\] something (re)",
            ),
        ];

        for (from, expect) in tests {
            let to = escape_misused_character_class(from);
            assert_eq!(expect, &to, "from `{}`", from);
        }
    }

    #[test]
    fn test_cleanup_unrecognized_escape_sequences() {
        let tests = vec![
            ("foo", "foo"),
            ("f\\^oo", "f\\^oo"),
            ("f\\$oo", "f\\$oo"),
            ("foo\\[bar\\]", "foo\\[bar\\]"),
            ("foo\\{bar\\}", "foo\\{bar\\}"),
            ("foo\\(bar\\)", "foo\\(bar\\)"),
            ("foo\\|bar", "foo\\|bar"),
            ("foo\\?bar", "foo\\?bar"),
            ("foo\\*bar", "foo\\*bar"),
            ("foo\\+bar", "foo\\+bar"),
            ("foo\\.bar", "foo\\.bar"),
            ("foo\\-bar", "foo\\-bar"),
            ("foo\\w\\W", "foo\\w\\W"),
            ("foo\\_bar", "foo_bar"),
            ("foo\\<bar\\>", "foo<bar>"),
        ];

        for (from, expect) in tests {
            let to = cleanup_unrecognized_escape_sequences(from);
            assert_eq!(expect, &to, "from `{}`", from);
        }
    }
}
