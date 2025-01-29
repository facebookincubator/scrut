/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Display;

use anyhow::Result;

use super::rule::Rule;
use super::rule::RuleMaker;

/// Simple equality match for lines that end in a new-line character
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EqualNoEolRule(String);

impl Display for EqualNoEolRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Rule for EqualNoEolRule {
    fn kind(&self) -> &'static str {
        "no-eol"
    }

    fn matches(&self, line: &[u8]) -> bool {
        self.0.as_bytes() == line
    }

    fn unmake(&self) -> (String, Vec<u8>) {
        (self.kind().to_string(), self.0.as_bytes().to_vec())
    }
}

impl RuleMaker for EqualNoEolRule {
    fn make(expression: &str) -> Result<Box<dyn Rule>> {
        Ok(Box::new(EqualNoEolRule(expression.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::EqualNoEolRule;
    use crate::lossy_string;
    use crate::newline::StringNewline;
    use crate::rules::rule::RuleMaker;

    #[test]
    fn test_make_unmake() {
        let rule = EqualNoEolRule::make("foo").expect("rule is created");
        let (kind, expression) = rule.unmake();
        assert_eq!("no-eol", kind);
        assert_eq!("foo", lossy_string!(&expression));
    }

    #[test]
    fn test_rule_matches() {
        let tests = [(true, "foo", "foo".to_string()),
            (false, "foo", "foo".assure_newline().to_string())];

        tests.iter().for_each(|(expect, expression, line)| {
            let rule = EqualNoEolRule::make(expression)
                .unwrap_or_else(|_| panic!("create rule from {}", expression));
            assert_eq!(
                *expect,
                rule.matches(line.as_bytes()),
                "with input `{}`",
                line
            )
        });
    }

    #[test]
    fn test_rule_serialize() {
        let rule = EqualNoEolRule::make("abc").unwrap();
        let serialized = serde_json::to_string(&rule).expect("serialize");
        assert_eq!("{\"kind\":\"no-eol\",\"expression\":\"abc\"}", serialized);
    }
}
