/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BTreeMap;

use anyhow::Result;
use regex::Regex;

use crate::expectation::Expectation;
use crate::expectation::ExpectationMaker;
use crate::rules::registry::RuleRegistry;

/// Interpolate `$VAR` and `${VAR}` references in `text` using values from `env`.
///
/// Rules:
/// - `$IDENTIFIER` is replaced with the value (IDENTIFIER = [A-Za-z_][A-Za-z0-9_]*)
/// - `${IDENTIFIER}` is also replaced
/// - `$$` produces a literal `$`
/// - If a variable is not found in `env`, the original text is left unchanged
pub fn interpolate_str(text: &str, env: &BTreeMap<String, String>) -> String {
    lazy_static! {
        static ref VAR_RE: Regex = Regex::new(
            r"(?x)
            \$                          # start with a $
            (
                \$ |                    # escaped $
                \{([A-Za-z0-9]+)\} |    # wrapped variable ${VAR}
                ([A-Za-z0-9_]+)         # plain variable $VAR
            )"
        )
        .unwrap();
    }

    VAR_RE
        .replace_all(text, |caps: &regex::Captures| {
            if &caps[1] == "$" {
                return '$'.to_string(); // $$ -> $
            }
            let name = caps.get(2).or_else(|| caps.get(3)).unwrap().as_str();
            env // $VAR or ${VAR} -> env[VAR]
                .get(name)
                .map(|s| s as &str)
                .unwrap_or_else(|| caps.get_match().as_str())
                .to_string()
        })
        .into_owned()
}

/// Interpolate the expression inside an expectation, returning a new
/// Expectation with the same kind/quantifier but an interpolated expression.
pub fn interpolate_expectation(
    expectation: &Expectation,
    env: &BTreeMap<String, String>,
) -> Result<Expectation> {
    let (kind, expression_bytes, optional, multiline) = expectation.unmake();
    let expression = String::from_utf8_lossy(&expression_bytes);
    let interpolated_expr = interpolate_str(&expression, env);
    if interpolated_expr == expression.as_ref() {
        return Ok(expectation.clone());
    }
    let maker = ExpectationMaker::new(RuleRegistry::default());
    maker.make(
        &kind,
        &interpolated_expr,
        optional,
        multiline,
        &expectation.original_string(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::interpolate_expectation;
    use super::interpolate_str;
    use crate::test_expectation;

    fn make_env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_basic_substitution() {
        let env = make_env(&[("FOO", "bar")]);
        assert_eq!(interpolate_str("Hello $FOO", &env), "Hello bar");
    }

    #[test]
    fn test_braced_syntax() {
        let env = make_env(&[("FOO", "bar")]);
        assert_eq!(interpolate_str("Hello ${FOO}", &env), "Hello bar");
    }

    #[test]
    fn test_missing_variable_stays() {
        let env = BTreeMap::new();
        assert_eq!(interpolate_str("Hello $MISSING", &env), "Hello $MISSING");
    }

    #[test]
    fn test_missing_braced_variable_stays() {
        let env = BTreeMap::new();
        assert_eq!(
            interpolate_str("Hello ${MISSING}", &env),
            "Hello ${MISSING}"
        );
    }

    #[test]
    fn test_escaped_dollar() {
        let env = BTreeMap::new();
        assert_eq!(interpolate_str("Cost is $$5", &env), "Cost is $5");
        assert_eq!(
            interpolate_str("This is $$AAA and $$BB", &env),
            "This is $AAA and $BB"
        );
    }

    #[test]
    fn test_adjacent_variables() {
        let env = make_env(&[("A", "x"), ("B", "y")]);
        assert_eq!(interpolate_str("$A$B", &env), "xy");
    }

    #[test]
    fn test_variable_at_end() {
        let env = make_env(&[("VAR", "value")]);
        assert_eq!(interpolate_str("prefix $VAR", &env), "prefix value");
    }

    #[test]
    fn test_dollar_at_end() {
        let env = BTreeMap::new();
        assert_eq!(interpolate_str("end$", &env), "end$");
    }

    #[test]
    fn test_non_identifier_after_dollar() {
        let env = BTreeMap::new();
        assert_eq!(interpolate_str("$123", &env), "$123");
    }

    #[test]
    fn test_empty_braced_name() {
        let env = BTreeMap::new();
        assert_eq!(interpolate_str("${}", &env), "${}");
    }

    #[test]
    fn test_no_variables() {
        let env = make_env(&[("FOO", "bar")]);
        assert_eq!(interpolate_str("no vars here", &env), "no vars here");
    }

    #[test]
    fn test_interpolate_expectation_equal() {
        let env = make_env(&[("FOO", "bar")]);
        let exp = test_expectation!("equal", "Hello $FOO");
        let result = interpolate_expectation(&exp, &env).expect("interpolate");
        assert!(result.matches(b"Hello bar\n"));
    }

    #[test]
    fn test_interpolate_expectation_glob() {
        let env = make_env(&[("PREFIX", "Hello")]);
        let exp = test_expectation!("glob", "$PREFIX*");
        let result = interpolate_expectation(&exp, &env).expect("interpolate");
        assert!(result.matches(b"Hello World\n"));
    }

    #[test]
    fn test_interpolate_expectation_regex() {
        let env = make_env(&[("WORD", "Hello")]);
        let exp = test_expectation!("regex", "$WORD.*");
        let result = interpolate_expectation(&exp, &env).expect("interpolate");
        assert!(result.matches(b"Hello World\n"));
    }

    #[test]
    fn test_interpolate_expectation_noop_no_vars() {
        let env = make_env(&[("FOO", "bar")]);
        let exp = test_expectation!("equal", "no vars here");
        let result = interpolate_expectation(&exp, &env).expect("interpolate");
        assert_eq!(result, exp);
    }

    #[test]
    fn test_interpolate_expectation_noop_empty_env() {
        let env = BTreeMap::new();
        let exp = test_expectation!("equal", "Hello $FOO");
        let result = interpolate_expectation(&exp, &env).expect("interpolate");
        // No substitution happened since $FOO not in env, expression unchanged
        assert_eq!(result, exp);
    }
}
