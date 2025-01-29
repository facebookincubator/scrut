/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Debug;
use std::fmt::Display;

use anyhow::Result;
use serde::Serialize;

use super::rule::Rule;
use super::rule::RuleMaker;
use crate::newline::BytesNewline;

/// Simple equality match for lines that end in a new-line character
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EqualRule(String);

impl Display for EqualRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Rule for EqualRule {
    fn kind(&self) -> &'static str {
        "equal"
    }

    fn matches(&self, line: &[u8]) -> bool {
        self.0.as_bytes().assure_newline() == line
    }

    fn unmake(&self) -> (String, Vec<u8>) {
        (self.kind().to_string(), self.0.as_bytes().to_vec())
    }
}

impl RuleMaker for EqualRule {
    fn make(expression: &str) -> Result<Box<dyn Rule>> {
        Ok(Box::new(EqualRule(expression.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::EqualRule;
    use crate::lossy_string;
    use crate::newline::StringNewline;
    use crate::rules::rule::RuleMaker;

    #[test]
    fn test_make_unmake() {
        let rule = EqualRule::make("foo").expect("rule is created");
        let (kind, expression) = rule.unmake();
        assert_eq!("equal", kind);
        assert_eq!("foo", lossy_string!(&expression));
    }

    #[test]
    fn test_rule_matches() {
        let tests = [(false, "foo", "foo".to_string()),
            (true, "foo", "foo".assure_newline().to_string())];

        tests.iter().for_each(|(expect, expression, line)| {
            let rule = EqualRule::make(expression)
                .unwrap_or_else(|_| panic!("create rule from {}", expression));
            assert_eq!(*expect, rule.matches(line.as_bytes()),)
        });
    }

    #[test]
    fn test_rule_serialize() {
        let rule = EqualRule::make("abc").unwrap();
        let serialized = serde_json::to_string(&rule).expect("serialize");
        assert_eq!("{\"kind\":\"equal\",\"expression\":\"abc\"}", serialized);
    }
}
