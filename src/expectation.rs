/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Display;

use anyhow::Result;
use serde::Serialize;

use crate::escaping::Escaper;
use crate::newline::StringNewline;
use crate::rules::registry::RuleRegistry;
use crate::rules::rule::Rule;

/// An expectation about the content and / or form of one or multiple subsequent
/// line(s) of output, that may be optional.
#[derive(Debug, Clone)]
pub struct Expectation {
    /// Optional Expectations are lines that may or may not occur in the output
    pub optional: bool,

    /// Multiline Expectations (can) match multiple sequential lines of output
    pub multiline: bool,

    /// The actual algorithm that implements the Expectation
    pub rule: Box<dyn Rule>,

    /// The original expression as it was written in the test file
    original: String,
}

impl Expectation {
    /// Decompose the Expectation into the components from which it can be made (`Expectation::make`)
    pub fn unmake(&self) -> (String, Vec<u8>, bool, bool) {
        let (kind, expression) = self.rule.unmake();
        (kind, expression, self.optional, self.multiline)
    }

    /// Whether the provided line matches this Expectation
    pub fn matches(&self, line: &[u8]) -> bool {
        self.rule.matches(line)
    }

    /// Renders the Expectation into an expression from which it can be parsed
    pub fn to_expression_string(&self, escaper: &Escaper) -> String {
        self.rule
            .to_expression_string(self.optional, self.multiline, escaper)
    }

    /// The original string as it was written in the test file
    pub fn original_string(&self) -> String {
        self.original.clone()
    }
}

impl Display for Expectation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_expression_string(&Escaper::default()))
    }
}

impl PartialEq for Expectation {
    fn eq(&self, other: &Self) -> bool {
        self.optional == other.optional
            && self.multiline == other.multiline
            && self.rule.to_string() == other.rule.to_string()
    }
}

impl Serialize for Expectation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.original)
    }
}

/// Facade for [`Expectation`] creation from either line encoded representation
/// or from components
pub struct ExpectationMaker(RuleRegistry);

impl ExpectationMaker {
    pub fn new(registry: RuleRegistry) -> Self {
        Self(registry)
    }

    // TODO: T131320483 following sentence doesnt make sense
    /// Create an [`Expectation`] that from it's text encoding, with the BNF form:
    ///
    /// ```bnf
    ///  <expectation> ::= <expression> | <expression> (<kind>) | <expression> (<quantifier>) | <expression> (<kind><quantifier>)
    ///   <expression> ::= "arbitrary text"
    ///         <kind> ::= <equal-kind> | <no-eol-kind> | <escaped-kind> | <glob-kind> | <regex-kind>
    ///   <equal-kind> ::= "equal" | "eq"
    ///  <no-eol-kind> ::= "no-eol"
    /// <escaped-kind> ::= "escaped" | "esc"
    ///    <glob-kind> ::= "glob" | "gl"
    ///   <regex-kind> ::= "regex" | "re"
    ///   <quantifier> ::= "?" | "*" | "+"
    /// ```
    ///
    /// ```
    /// use scrut::expectation::ExpectationMaker;
    /// use scrut::rules::registry::RuleRegistry;
    ///
    /// let maker = ExpectationMaker::new(RuleRegistry::default());
    /// maker.parse("foo bar").expect("parses expectation");
    /// maker
    ///     .parse("^foo bar$ (regex)")
    ///     .expect("parses expectation");
    /// ```
    pub fn parse(&self, line: &str) -> Result<Expectation> {
        let (expression, kind, quantifier) = self.extract(line)?;
        let multiline = quantifier == "*" || quantifier == "+";
        let optional = quantifier == "*" || quantifier == "?";
        self.make(
            &kind,
            &expression,
            optional,
            multiline,
            &(&line).trim_newlines(),
        )
    }

    /// Create an [`Expectation`] from the components that make it up
    pub(crate) fn make(
        &self,
        kind: &str,
        expression: &str,
        optional: bool,
        multiline: bool,
        original: &str,
    ) -> Result<Expectation> {
        Ok(Expectation {
            optional,
            multiline,
            rule: self.0.make(kind, expression)?,
            original: original.into(),
        })
    }

    // TODO: rename return type so that people can understand
    fn extract(&self, line: &str) -> Result<(String, String, String)> {
        let captures = self
            .0
            .to_expectation_regex()?
            .captures(line)
            .map_or(vec![], |captures| {
                captures
                    .iter()
                    .skip(1)
                    .filter_map(|m| m.map(|v| v.as_str()))
                    .collect::<Vec<_>>()
            });
        if captures.len() == 1 {
            Ok((line.to_string(), "equal".to_string(), "".to_string()))
        } else if captures.len() == 2 {
            Ok((
                captures[0].to_string(),
                captures[1].to_string(),
                "".to_string(),
            ))
        } else {
            Ok((
                captures[0].to_string(),
                match captures[1] {
                    "" => "equal",
                    v => v,
                }
                .to_string(),
                captures[2].to_string(),
            ))
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::ExpectationMaker;
    use crate::escaping::Escaper;
    use crate::rules::registry::RuleRegistry;

    #[test]
    fn test_expectation_extract() {
        let tests = vec![
            ("foo", ("foo", "equal", "")),
            ("foo (?)", ("foo", "equal", "?")),
            ("foo (*)", ("foo", "equal", "*")),
            ("foo (+)", ("foo", "equal", "+")),
            ("foo (eq+)", ("foo", "eq", "+")),
            ("foo (equal+)", ("foo", "equal", "+")),
            ("foo (no-eol)", ("foo", "no-eol", "")),
            ("foo (no-eol?)", ("foo", "no-eol", "?")),
            ("foo (no-eol*)", ("foo", "no-eol", "*")),
            ("foo (no-eol+)", ("foo", "no-eol", "+")),
            ("foo (esc)", ("foo", "esc", "")),
            ("foo (esc*)", ("foo", "esc", "*")),
            ("foo (escaped)", ("foo", "escaped", "")),
            ("foo (escaped+)", ("foo", "escaped", "+")),
            ("foo (re)", ("foo", "re", "")),
            ("foo (re?)", ("foo", "re", "?")),
            ("foo (regex*)", ("foo", "regex", "*")),
            ("foo (regex+)", ("foo", "regex", "+")),
            ("foo (glob)", ("foo", "glob", "")),
            ("foo (glob?)", ("foo", "glob", "?")),
            ("foo (glob*)", ("foo", "glob", "*")),
            ("foo (glob+)", ("foo", "glob", "+")),
            ("foo (glob+) (glob+)", ("foo (glob+)", "glob", "+")),
        ];

        tests.iter().for_each(
            |(line, (expect_expression, expect_kind, expect_quantifier))| {
                let (expression, kind, quantifier) = expectation_maker()
                    .extract(line)
                    .expect("extract expression from line");
                assert_eq!(
                    expect_expression.to_string(),
                    expression,
                    "expression from '{line}'"
                );
                assert_eq!(expect_kind.to_string(), kind, "kind from '{line}'");
                assert_eq!(
                    expect_quantifier.to_string(),
                    quantifier,
                    "quantifier from '{line}'"
                );
            },
        );
    }

    #[test]
    fn test_parse_to_expression_string() {
        let tests = vec![
            ("foo", "foo"),
            ("foo (?)", "foo (?)"),
            ("foo (equal)", "foo"),
            ("foo (eq)", "foo"),
            ("foo (equal*)", "foo (*)"),
            ("foo (no-eol)", "foo (no-eol)"),
            ("foo (escaped)", "foo (escaped)"),
            ("foo (esc)", "foo (escaped)"),
            ("foo (esc+)", "foo (escaped+)"),
            ("foo (glob)", "foo (glob)"),
            ("foo (gl)", "foo (glob)"),
            ("foo (glob?)", "foo (glob?)"),
            ("foo (regex)", "foo (regex)"),
            ("foo (re)", "foo (regex)"),
            ("foo (regex*)", "foo (regex*)"),
        ];
        for (from, to) in tests {
            let expectation = expectation_maker()
                .parse(from)
                .unwrap_or_else(|_| panic!("parse `{from}`"));
            let rendered = expectation.to_expression_string(&Escaper::default());
            assert_eq!(rendered, *to, "`{from}` rendered back to `{to}`");
        }
    }

    pub(crate) fn expectation_maker() -> ExpectationMaker {
        ExpectationMaker::new(RuleRegistry::default())
    }

    #[macro_export]
    macro_rules! test_expectation {
        ($expression:expr) => {
            $crate::expectation::tests::expectation_maker()
                .make("equal", $expression, false, false, $expression)
                .expect("create test expectation")
        };
        ($kind:expr, $expression:expr) => {
            $crate::expectation::tests::expectation_maker()
                .make(
                    $kind,
                    $expression,
                    false,
                    false,
                    &format!("{} ({})", $expression, $kind),
                )
                .expect("create test expectation")
        };
        ($kind:expr, $expression:expr, $optional:expr, $multiline:expr) => {
            $crate::expectation::tests::expectation_maker()
                .make(
                    $kind,
                    $expression,
                    $optional,
                    $multiline,
                    &format!("{} ({})", $expression, $kind),
                )
                .expect("create test expectation")
        };
        ($kind:expr, $expression:expr, $optional:expr, $multiline:expr, $original:expr) => {
            $crate::expectation::tests::expectation_maker()
                .make($kind, $expression, $optional, $multiline, $original)
                .expect("create test expectation")
        };
    }
}
