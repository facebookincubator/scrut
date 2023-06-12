use std::collections::HashMap;

use anyhow::anyhow;
use anyhow::Result;
use regex::Regex;

use super::equal::EqualRule;
use super::escaped::EscapedRule;
use super::glob::GlobRule;
use super::no_eol::EqualNoEolRule;
use super::regex::RegexRule;
use super::rule::MakeRule;
use super::rule::Rule;
use super::rule::RuleMaker;

/// Registry for [`Rule`] constructors, that is used by the [`crate::expectation::ExpectationMaker`]
pub struct RuleRegistry {
    makers: HashMap<String, MakeRule>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self {
            makers: HashMap::new(),
        }
    }

    /// Crate a regular expression that matches a line that contains an
    /// [`crate::expectation::Expectation`], based on all registered rules
    /// matchers and their aliases
    pub(crate) fn to_expectation_regex(&self) -> Result<Regex> {
        let mut names = self.makers.keys().collect::<Vec<_>>();
        names.sort();
        let names = names
            .iter()
            .map(|name| regex::escape(name))
            .collect::<Vec<_>>()
            .join("|");
        let expression = format!(
            r"(?x)
            ^
            (.*?)
            (?:
                \s
                \(
                    (
                        {names}|
                    )?
                    ([*+?])?
                \)
            )?
            $
        "
        );
        Regex::new(&expression).map_err(anyhow::Error::new)
    }

    /// File a [`Rule`] constructor under given name
    pub fn register(&mut self, maker: MakeRule, names: &[&str]) -> &mut Self {
        for name in names {
            self.makers.insert(name.to_string(), maker);
        }
        self
    }

    /// Construct a [`Rule`] of the given kind (=name)
    pub fn make(&self, kind: &str, expression: &str) -> Result<Box<dyn Rule>> {
        if let Some(ref maker) = self.makers.get(kind) {
            maker(expression)
        } else {
            Err(anyhow!("no rule maker for {} registered", kind))
        }
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry
            .register(EqualRule::make, &["equal", "eq"])
            .register(EqualNoEolRule::make, &["no-eol"])
            .register(EscapedRule::make, &["escaped", "esc"])
            .register(GlobRule::make, &["glob", "gl"])
            .register(RegexRule::make, &["regex", "re"]);
        registry
    }
}

#[cfg(test)]
mod tests {

    use super::RuleRegistry;
    use crate::lossy_string;

    #[test]
    fn test_default() {
        let tests = vec![
            (vec!["equal", "eq"], "foo"),
            (vec!["no-eol"], "foo"),
            (vec!["escaped", "esc"], "foo"),
            (vec!["glob", "gl"], "foo"),
            (vec!["regex", "re"], "foo"),
        ];
        let registry = RuleRegistry::default();
        for (kinds, expression) in tests {
            let long_kind = kinds[0];
            for kind in kinds {
                let rule = registry
                    .make(kind, expression)
                    .unwrap_or_else(|_| panic!("make from `{kind}` `{expression}`"));
                let (to_kind, to_expression) = rule.unmake();
                assert_eq!(to_kind, long_kind, "resulting kind {kind}::{expression}");
                assert_eq!(
                    lossy_string!(&to_expression),
                    expression,
                    "resulting expression from {kind}::{expression}"
                );
            }
        }
    }
}
