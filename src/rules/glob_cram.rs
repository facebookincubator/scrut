use std::fmt::Display;

use anyhow::Context;
use anyhow::Result;
use regex::bytes::Regex;

use super::escaped_filter::apply_escaped_filter_utf8;
use super::escaped_filter::expression_as_escaped;
use super::rule::Rule;
use super::rule::RuleMaker;
use crate::newline::BytesNewline;

/// Simple equality match for lines that end in a new-line character
#[derive(Clone, Debug)]
pub struct CramGlobRule(String, Regex);

impl Display for CramGlobRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for CramGlobRule {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Rule for CramGlobRule {
    fn kind(&self) -> &'static str {
        "glob"
    }

    fn matches(&self, line: &[u8]) -> bool {
        self.1.is_match(line.trim_newlines())
    }

    fn unmake(&self) -> (String, Vec<u8>) {
        (self.kind().to_string(), self.0.as_bytes().to_vec())
    }
}

impl RuleMaker for CramGlobRule {
    fn make(expression: &str) -> Result<Box<dyn Rule>> {
        let expression = if let Some(expression) = expression_as_escaped(expression) {
            apply_escaped_filter_utf8(expression)?
        } else {
            expression.to_string()
        };
        let regex = glob_to_regex(&expression)?;
        Ok(Box::new(Self(expression, regex)))
    }
}

fn glob_to_regex(glob: &str) -> Result<Regex> {
    let expression = glob_to_regex_string(glob);
    Regex::new(&expression).context("glob to regex")
}

fn glob_to_regex_string(glob: &str) -> String {
    let mut result = String::new();
    let chars = glob.chars().collect::<Vec<_>>();
    let len = chars.len();
    let mut index = 0;
    while index < len {
        let ch = chars[index];
        index += 1;
        match ch {
            '\\' if index < len && ['*', '?', '\\'].contains(&chars[index]) => {
                result.push(ch);
                result.push(chars[index]);
                index += 1;
            }
            '*' => result.push_str(".*"),
            '?' => result.push('.'),
            _ => result.push_str(&regex::escape(ch.to_string().as_str())),
        }
    }
    format!("^{result}$")
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use regex::Regex;

    use super::glob_to_regex;
    use super::glob_to_regex_string;
    use super::CramGlobRule;
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
            let rule = CramGlobRule::make(from)
                .unwrap_or_else(|_| panic!("rule is created from `{from}`"));
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
            (true, "foo\\?bar", "foo?bar".to_string()),
            (false, "foo\\?bar", "fooXbar".to_string()),
            (true, "foo\\*bar*", "foo*bar".to_string()),
            (false, "foo\\*bar*", "fooXbar".to_string()),
            (true, "foo\\*bar*", "foo*barbaz".to_string()),
            (true, "foo\\tbar* (esc)", "foo\tbarbaz".to_string()),
            (true, "foo\\x1d[1mbar* (esc)", "foo\x1d[1mbarbaz".to_string()),
            (
                true,
                r#""description": "`*foo* with code 1\n--- stdout\nthis is stdout\n--- stderr\nthis is stderr""#,
                r#""description": "`/tmp/execution.123123/bla/test/foo --bar --baz with code 1\n--- stdout\nthis is stdout\n--- stderr\nthis is stderr""#.to_string(),
            ),
        ];

        tests.iter().for_each(|(expect, expression, line)| {
            let rule = CramGlobRule::make(expression)
                .with_context(|| format!("make glob rule from {expression}"))
                .unwrap();
            assert_eq!(
                *expect,
                rule.matches(line.as_bytes()),
                "glob `{expression}` matches `{line}`",
            )
        })
    }

    #[test]
    fn test_rule_serialize() {
        let rule = CramGlobRule::make("abc").unwrap();
        let serialized = serde_json::to_string(&rule).expect("serialize");
        assert_eq!("{\"kind\":\"glob\",\"expression\":\"abc\"}", serialized);
    }

    #[test]
    fn test_glob_to_regex_string() {
        [
            ("foo", "^foo$"),
            ("foo*", "^foo.*$"),
            ("foo?", "^foo.$"),
            ("foo\\*", "^foo\\*$"),
            ("foo\\**", "^foo\\*.*$"),
            ("[fo]o", "^\\[fo\\]o$"),
        ]
        .iter()
        .for_each(|(from, expect)| {
            let to = glob_to_regex_string(from);
            assert_eq!(*expect, to.as_str(), "from '{}' get '{}'", from, expect)
        })
    }

    #[test]
    fn test_glob_to_regex() {
        [
            ("foo", Regex::new("^foo$").unwrap()),
            ("foo*", Regex::new("^foo.*$").unwrap()),
            ("foo?", Regex::new("^foo.$").unwrap()),
            ("foo\\*", Regex::new("^foo\\*$").unwrap()),
            ("foo\\**", Regex::new("^foo\\*.*$").unwrap()),
        ]
        .iter()
        .for_each(|(from, expect)| {
            let to = glob_to_regex(from).expect("compiles");
            assert_eq!(
                expect.to_string(),
                to.to_string(),
                "from '{from}' get '{expect}'"
            )
        })
    }
}
