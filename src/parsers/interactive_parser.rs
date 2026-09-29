/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Parser for interactive test body lines.
//!
//! Converts lines after the `$` command in an interactive-mode test case into
//! [`InteractiveDirective`]s. Supports both long-form (`WAIT:`, `SEND_KEYS:`,
//! `ASSERT:`, `WRITE:`) and short-form (`@`, `^`, `!`, `|`) syntax.
//!
//! Directive parameters can be specified in two equivalent ways:
//!
//! 1. **Inline** — `DIRECTIVE {key: value}: content`
//! 2. **Multiline** — `%` body lines preceding the directive:
//!    ```text
//!    % timeout: 5s
//!    @ pattern
//!    ```
//!
//! Both forms are parsed into a parser-internal `DirectiveParams`, then
//! extracted into per-variant fields on `InteractiveDirective`. When both
//! are present, inline values override multiline values.
//!
//! Per-directive validation restricts which parameters are allowed:
//! - `WAIT` / `@`: `timeout`
//! - `SEND_KEYS` / `^`: `if_match`, `unless_match`
//! - `ASSERT` / `!`: none
//! - `WRITE` / `|`: none
//!
//! (Not to be confused with `%` in config position before the `$` command,
//! which is multiline YAML config.)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::expectation::ExpectationMaker;
use crate::interactive::InteractiveDirective;
use crate::interactive::InteractivePattern;
use crate::interactive::PatternKind;
use crate::interactive::SendKeysCondition;

/// Parser-internal intermediate representation for directive parameters.
///
/// Carries optional parameters parsed from inline `{...}` syntax or multiline
/// `%` body lines. Per-directive validation restricts which parameters are
/// allowed for each directive kind. Fields are extracted into the appropriate
/// enum variant fields when constructing `InteractiveDirective`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DirectiveParams {
    /// Per-directive timeout override (WAIT only).
    timeout: Option<Duration>,
    /// Conditional execution (SEND_KEYS only).
    condition: Option<SendKeysCondition>,
}

impl DirectiveParams {
    /// Merge two parameter blocks. Values from `other` take precedence.
    fn merge(self, other: DirectiveParams) -> DirectiveParams {
        DirectiveParams {
            timeout: other.timeout.or(self.timeout),
            condition: other.condition.or(self.condition),
        }
    }
}

/// Parses body lines in interactive mode into [`InteractiveDirective`]s.
///
/// Handles both long-form and short-form syntax, plus `%` parameter lines
/// that are buffered and applied to the next directive.
pub(crate) struct InteractiveBodyParser {
    expectation_maker: Arc<ExpectationMaker>,
    /// Accumulated `%` parameter lines (stripped of `% ` prefix), joined as
    /// YAML and parsed into `DirectiveParams` when the next directive arrives.
    pending_param_lines: Vec<String>,
}

impl InteractiveBodyParser {
    pub fn new(expectation_maker: Arc<ExpectationMaker>) -> Self {
        Self {
            expectation_maker,
            pending_param_lines: Vec::new(),
        }
    }

    /// Parse a body line into an interactive directive.
    ///
    /// Returns `None` for parameter-modifier lines (`%`), which are buffered
    /// internally and applied to the next directive.
    pub fn parse_line(&mut self, line: &str, index: usize) -> Result<Option<InteractiveDirective>> {
        let line_number = index + 1;

        macro_rules! read_params {
            ($name:expr, $rest:expr, $allow_params:expr) => {{
                let pending = self.take_pending_params(line_number)?;
                let (params, content) = if $allow_params {
                    let (inline_params, content) =
                        extract_params_and_content($rest, line_number)
                            .with_context(|| format!("parsing {} directive", $name))?;
                    (pending.merge(inline_params), content)
                } else {
                    (pending, $rest)
                };
                validate_params(&params, $name, line_number)?;
                (params, content)
            }};
        }

        if let Some((rest, allow_params)) = strip_keyword_or_symbol(line, "WAIT", "@") {
            let (params, content) = read_params!("WAIT", rest, allow_params);
            return Ok(Some(InteractiveDirective::Wait {
                pattern: self.parse_pattern(content)?,
                line_number,
                timeout: params.timeout,
            }));
        }

        if let Some((rest, allow_params)) = strip_keyword_or_symbol(line, "SEND_KEYS", "^") {
            let (params, content) = read_params!("SEND_KEYS", rest, allow_params);
            let keys = parse_key_list(content)
                .with_context(|| format!("line {}: parsing SEND_KEYS", line_number))?;
            return Ok(Some(InteractiveDirective::SendKeys {
                keys,
                condition: params.condition,
                line_number,
            }));
        }

        if let Some((rest, allow_params)) = strip_keyword_or_symbol(line, "ASSERT", "!") {
            let (_, content) = read_params!("ASSERT", rest, allow_params);
            return Ok(Some(InteractiveDirective::Assert {
                pattern: self.parse_pattern(content)?,
                line_number,
            }));
        }

        if let Some((rest, allow_params)) = strip_keyword_or_symbol(line, "WRITE", "|") {
            let (_, content) = read_params!("WRITE", rest, allow_params);
            return Ok(Some(InteractiveDirective::Write {
                text: content.to_string(),
                line_number,
            }));
        }

        // Parameter modifier lines (% in body position for interactive mode)
        if let Some(rest) = line.strip_prefix("% ") {
            self.pending_param_lines.push(rest.to_string());
            return Ok(None);
        } else if line.starts_with("%") {
            // ignore this case, allow for empty config lines in body
            return Ok(None);
        }

        bail!(
            "line {}: unrecognized interactive directive: {:?}",
            line_number,
            line
        )
    }

    /// Verify no pending parameter lines were left unconsumed at end of testcase.
    pub fn finish(&self) -> Result<()> {
        if !self.pending_param_lines.is_empty() {
            bail!("trailing parameter lines without a following directive")
        }
        Ok(())
    }

    /// Reset parser state for the next testcase.
    pub fn reset(&mut self) {
        self.pending_param_lines.clear();
    }

    /// Parse a pattern string using the existing ExpectationMaker's rule suffix syntax.
    ///
    /// Extracts the rule kind from `(regex)`, `(glob)`, etc. suffix (including
    /// short aliases like `(re)`); if no suffix, defaults to equal.
    fn parse_pattern(&self, text: &str) -> Result<InteractivePattern> {
        let expectation = self
            .expectation_maker
            .parse(text)
            .context("parsing interactive pattern")?;
        let kind: PatternKind = expectation
            .rule
            .kind()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}: {}", text, err))
            .context("parsing interactive pattern kind")?;
        let expression = expectation.expression();
        Ok(InteractivePattern { expression, kind })
    }

    /// Consume accumulated `%` parameter lines, parse them as YAML-like
    /// key-value pairs into `DirectiveParams`, and clear the buffer.
    fn take_pending_params(&mut self, line_number: usize) -> Result<DirectiveParams> {
        if self.pending_param_lines.is_empty() {
            return Ok(DirectiveParams::default());
        }

        let yaml_str = self.pending_param_lines.join("\n");
        self.pending_param_lines.clear();
        parse_params_yaml(&yaml_str)
            .with_context(|| format!("line {}: parsing parameter lines", line_number))
    }
}

/// Parse a YAML-like string into `DirectiveParams`.
///
/// Recognized keys:
/// - `timeout` — parsed as a duration (e.g., `5s`, `500ms`)
/// - `if_match` — parsed as a MATCHER, creates `IfMatch` condition
/// - `unless_match` — parsed as a MATCHER, creates `UnlessMatch` condition
///
/// Any unrecognized key produces an error.
fn parse_params_yaml(yaml_str: &str) -> Result<DirectiveParams> {
    let map: HashMap<String, String> =
        serde_yaml::from_str(yaml_str).context("invalid YAML in directive parameters")?;

    let mut params = DirectiveParams::default();
    for (key, value) in &map {
        match key.as_str() {
            "timeout" => {
                if params.timeout.is_some() {
                    bail!("duplicate 'timeout' parameter");
                }
                params.timeout = Some(parse_duration(value)?);
            }
            "if_match" => {
                if params.condition.is_some() {
                    bail!(
                        "duplicate if_match parameter (only one if_match or unless_match allowed)"
                    );
                }
                let pattern = parse_matcher(value)?;
                params.condition = Some(SendKeysCondition::IfMatch(pattern));
            }
            "unless_match" => {
                if params.condition.is_some() {
                    bail!(
                        "duplicate unless_match parameter (only one if_match or unless_match allowed)"
                    );
                }
                let pattern = parse_matcher(value)?;
                params.condition = Some(SendKeysCondition::UnlessMatch(pattern));
            }
            _ => {
                bail!("unrecognized directive parameter: {:?}", key);
            }
        }
    }
    Ok(params)
}

/// Match a directive keyword or symbol at the start of a line.
///
/// The keyword/symbol must be followed by ` `, `:`, or `{` to avoid false matches
/// (e.g., `WAITING` should not match `WAIT`). Returns the remainder after
/// the keyword (not including the delimiter character, except for `{` which
/// is kept for brace parsing).
fn strip_keyword_or_symbol<'a>(
    line: &'a str,
    keyword: &str,
    symbol: &str,
) -> Option<(&'a str, bool)> {
    if let Some(rest) = line.strip_prefix(keyword) {
        if rest.starts_with(':') {
            return Some((rest, true));
        }
        if let Some(brace_rest) = rest.strip_prefix(' ') {
            if brace_rest.starts_with('{') && brace_rest.contains("}:") {
                return Some((brace_rest, true));
            }
        }
    } else if let Some(rest) = line.strip_prefix(&format!("{symbol} ")) {
        return Some((rest, false));
    }
    None
}

/// Given everything after the directive keyword, extract inline `{...}` params
/// and the content string.
///
/// Handles both `{key: val}: content` and `: content` forms.
/// Returns `(DirectiveParams, &str)` — the parsed inline params and the
/// remaining content.
fn extract_params_and_content<'a>(
    rest: &'a str,
    line_number: usize,
) -> Result<(DirectiveParams, &'a str)> {
    // Case 1: {params}: content
    if let Some(brace_content) = rest.strip_prefix('{') {
        if let Some((params_str, content)) = brace_content.split_once("}: ") {
            let params = parse_params_yaml(params_str)
                .with_context(|| format!("line {}: parsing inline parameters", line_number))?;
            return Ok((params, content));
        }
        // Malformed braces — fall through to treat as plain content
    }

    // Case 2: : content  or  :content  (with optional space after colon)
    if let Some(content) = rest.strip_prefix(':') {
        return Ok((DirectiveParams::default(), content.trim_start()));
    }

    // Case 3: space then content (e.g., "WAIT foo" without colon — shouldn't
    // normally happen but handle gracefully as plain content)
    if let Some(content) = rest.strip_prefix(' ') {
        return Ok((DirectiveParams::default(), content));
    }

    bail!(
        "line {}: expected ':' or '{{' after directive keyword, got: {:?}",
        line_number,
        rest
    )
}

/// Validate that params are allowed for the given directive kind.
fn validate_params(params: &DirectiveParams, kind: &str, line_number: usize) -> Result<()> {
    let supported = match kind {
        "WAIT" => vec!["timeout"],
        "SEND_KEYS" => vec!["condition"],
        _ => vec![],
    };

    if params.condition.is_some() && !supported.contains(&"condition") {
        bail!(
            "line {}: 'if_match'/'unless_match' not allowed on {} directive",
            line_number,
            kind
        );
    }
    if params.timeout.is_some() && !supported.contains(&"timeout") {
        bail!(
            "line {}: 'timeout' not allowed on {} directive",
            line_number,
            kind
        );
    }
    Ok(())
}

/// Split key list on whitespace into individual key names, validating each token.
fn parse_key_list(text: &str) -> Result<Vec<String>> {
    text.split_whitespace()
        .map(|token| {
            validate_key_token(token)?;
            Ok(token.to_string())
        })
        .collect()
}

/// Validate that a SEND_KEYS token is syntactically valid.
///
/// Valid tokens: single character, hex byte (#xx / 0xxx), or key name (uppercase + digits/underscore/plus).
fn validate_key_token(token: &str) -> Result<()> {
    // Single character: always valid
    if token.chars().count() == 1 {
        return Ok(());
    }

    // Hex: #xx or 0xxx
    if let Some(hex) = token.strip_prefix('#').or_else(|| token.strip_prefix("0x")) {
        if !hex.is_empty() && hex.len() <= 2 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(());
        }
        bail!(
            "invalid hex key {:?}: expected 1-2 hex digits after prefix",
            token
        );
    }

    // Key name pattern: starts with uppercase, all uppercase/digits/underscore/plus
    if token.starts_with(|c: char| c.is_ascii_uppercase())
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '+')
    {
        return Ok(());
    }

    bail!(
        "invalid SEND_KEYS token {:?}: each token must be a single character \
         (e.g. a, 1), a named key (e.g. ENTER, DOWN, CTRL+C), or a hex value \
         (e.g. #0a, 0xff). To type a word, use WRITE: or spell out characters: \
         SEND_KEYS: {}",
        token,
        token
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    )
}

/// Parse a MATCHER expression:
/// - `/regex/` — regex match
/// - any other string — equal match
fn parse_matcher(text: &str) -> Result<InteractivePattern> {
    let text = text.trim();
    if text.starts_with('/') && text.ends_with('/') && text.len() >= 2 {
        let expression = text[1..text.len() - 1].to_string();
        Ok(InteractivePattern {
            expression,
            kind: PatternKind::Regex,
        })
    } else {
        Ok(InteractivePattern {
            expression: text.to_string(),
            kind: PatternKind::Equal,
        })
    }
}

/// Parse a human-readable duration string like "5s", "500ms", "1m", "1m30s".
fn parse_duration(text: &str) -> Result<Duration> {
    humantime::parse_duration(text)
        .map_err(|e| anyhow::anyhow!("invalid duration {:?}: {}", text, e))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::InteractiveBodyParser;
    use crate::expectation::tests::expectation_maker;
    use crate::interactive::InteractiveDirective;
    use crate::interactive::InteractivePattern;
    use crate::interactive::PatternKind;
    use crate::interactive::SendKeysCondition;

    fn parser() -> InteractiveBodyParser {
        InteractiveBodyParser::new(Arc::new(expectation_maker()))
    }

    #[test]
    fn test_parse_wait_long_form() {
        let mut p = parser();
        let dir = p.parse_line("WAIT: Select a flavor", 4).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "Select a flavor".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 5,
                timeout: None,
            }
        );
    }

    #[test]
    fn test_parse_wait_short_form() {
        let mut p = parser();
        let dir = p.parse_line("@ Select a flavor", 4).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "Select a flavor".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 5,
                timeout: None,
            }
        );
    }

    #[test]
    fn test_parse_wait_with_regex() {
        let mut p = parser();
        let dir = p
            .parse_line("WAIT: .*flavor.* (regex)", 4)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: ".*flavor.*".to_string(),
                    kind: PatternKind::Regex,
                },
                line_number: 5,
                timeout: None,
            }
        );
    }

    #[test]
    fn test_parse_wait_with_glob() {
        let mut p = parser();
        let dir = p.parse_line("WAIT: *flavor* (glob)", 4).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "*flavor*".to_string(),
                    kind: PatternKind::Glob,
                },
                line_number: 5,
                timeout: None,
            }
        );
    }

    #[test]
    fn test_parse_assert_long_form() {
        let mut p = parser();
        let dir = p
            .parse_line("ASSERT: .*something.* (regex)", 7)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Assert {
                pattern: InteractivePattern {
                    expression: ".*something.*".to_string(),
                    kind: PatternKind::Regex,
                },
                line_number: 8,
            }
        );
    }

    #[test]
    fn test_parse_assert_short_form() {
        let mut p = parser();
        let dir = p.parse_line("! .*something.* (regex)", 7).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Assert {
                pattern: InteractivePattern {
                    expression: ".*something.*".to_string(),
                    kind: PatternKind::Regex,
                },
                line_number: 8,
            }
        );
    }

    #[test]
    fn test_parse_send_keys_long_form() {
        let mut p = parser();
        let dir = p
            .parse_line("SEND_KEYS: DOWN DOWN DOWN ENTER", 5)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::SendKeys {
                keys: vec![
                    "DOWN".to_string(),
                    "DOWN".to_string(),
                    "DOWN".to_string(),
                    "ENTER".to_string()
                ],
                condition: None,
                line_number: 6,
            }
        );
    }

    #[test]
    fn test_parse_send_keys_short_form() {
        let mut p = parser();
        let dir = p.parse_line("^ DOWN ENTER", 5).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::SendKeys {
                keys: vec!["DOWN".to_string(), "ENTER".to_string()],
                condition: None,
                line_number: 6,
            }
        );
    }

    #[test]
    fn test_parse_send_keys_with_inline_if_match() {
        let mut p = parser();
        let dir = p
            .parse_line("SEND_KEYS {if_match: \"flavor\"}: DOWN", 5)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::SendKeys {
                keys: vec!["DOWN".to_string()],
                condition: Some(SendKeysCondition::IfMatch(InteractivePattern {
                    expression: "flavor".to_string(),
                    kind: PatternKind::Equal,
                })),
                line_number: 6,
            }
        );
    }

    #[test]
    fn test_parse_send_keys_with_inline_unless_match_regex() {
        let mut p = parser();
        let dir = p
            .parse_line("SEND_KEYS {unless_match: /.*flavor.*/}: DOWN", 5)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::SendKeys {
                keys: vec!["DOWN".to_string()],
                condition: Some(SendKeysCondition::UnlessMatch(InteractivePattern {
                    expression: ".*flavor.*".to_string(),
                    kind: PatternKind::Regex,
                })),
                line_number: 6,
            }
        );
    }

    #[test]
    fn test_parse_condition_modifier_short_form() {
        let mut p = parser();
        // % line returns None (buffered)
        let result = p.parse_line("% unless_match: \"flavor\"", 5).unwrap();
        assert!(result.is_none());

        // Next ^ line picks up the buffered condition
        let dir = p.parse_line("^ DOWN", 6).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::SendKeys {
                keys: vec!["DOWN".to_string()],
                condition: Some(SendKeysCondition::UnlessMatch(InteractivePattern {
                    expression: "flavor".to_string(),
                    kind: PatternKind::Equal,
                })),
                line_number: 7,
            }
        );
    }

    #[test]
    fn test_parse_condition_modifier_with_regex() {
        let mut p = parser();
        let result = p.parse_line("% if_match: /.*pattern.*/", 5).unwrap();
        assert!(result.is_none());

        let dir = p.parse_line("^ ENTER", 6).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::SendKeys {
                keys: vec!["ENTER".to_string()],
                condition: Some(SendKeysCondition::IfMatch(InteractivePattern {
                    expression: ".*pattern.*".to_string(),
                    kind: PatternKind::Regex,
                })),
                line_number: 7,
            }
        );
    }

    #[test]
    fn test_error_unrecognized_line() {
        let mut p = parser();
        let result = p.parse_line("some random text", 5);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("unrecognized interactive directive"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_trailing_params() {
        let mut p = parser();
        p.parse_line("% if_match: \"text\"", 5).unwrap();
        let result = p.finish();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("trailing parameter lines"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_parse_unquoted_matcher_value() {
        let mut p = parser();
        let result = p.parse_line("% if_match: no_quotes", 5);
        assert!(result.is_ok());
        // Buffered — force consumption
        let dir = p.parse_line("^ DOWN", 6).unwrap().unwrap();
        match dir {
            InteractiveDirective::SendKeys { condition, .. } => {
                let cond = condition.expect("condition should be set");
                match cond {
                    SendKeysCondition::IfMatch(pattern) => {
                        assert_eq!(pattern.expression, "no_quotes");
                        assert_eq!(pattern.kind, PatternKind::Equal);
                    }
                    _ => panic!("expected IfMatch"),
                }
            }
            _ => panic!("expected SendKeys"),
        }
    }

    #[test]
    fn test_parse_wait_with_inline_timeout() {
        let mut p = parser();
        let dir = p
            .parse_line("WAIT {timeout: 5s}: Select a flavor", 4)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "Select a flavor".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 5,
                timeout: Some(Duration::from_secs(5)),
            }
        );
    }

    #[test]
    fn test_parse_wait_with_inline_timeout_millis() {
        let mut p = parser();
        let dir = p
            .parse_line("WAIT {timeout: 500ms}: .*ready.* (regex)", 4)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: ".*ready.*".to_string(),
                    kind: PatternKind::Regex,
                },
                line_number: 5,
                timeout: Some(Duration::from_millis(500)),
            }
        );
    }

    #[test]
    fn test_parse_wait_with_pending_timeout() {
        let mut p = parser();
        // % timeout: 3s returns None (buffered)
        let result = p.parse_line("% timeout: 3s", 5).unwrap();
        assert!(result.is_none());

        // Next WAIT picks up the buffered timeout
        let dir = p.parse_line("WAIT: Select a flavor", 6).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "Select a flavor".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 7,
                timeout: Some(Duration::from_secs(3)),
            }
        );
    }

    #[test]
    fn test_parse_wait_short_form_with_pending_timeout() {
        let mut p = parser();
        let result = p.parse_line("% timeout: 2s", 5).unwrap();
        assert!(result.is_none());

        let dir = p.parse_line("@ Select a flavor", 6).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "Select a flavor".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 7,
                timeout: Some(Duration::from_secs(2)),
            }
        );
    }

    #[test]
    fn test_error_trailing_timeout() {
        let mut p = parser();
        p.parse_line("% timeout: 5s", 5).unwrap();
        let result = p.finish();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("trailing parameter lines"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_invalid_timeout_duration() {
        let mut p = parser();
        let result = p.parse_line("WAIT {timeout: not_a_duration}: foo", 5);
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(err_msg.contains("invalid duration"), "error: {}", err_msg);
    }

    #[test]
    fn test_error_unrecognized_param_key() {
        let mut p = parser();
        let result = p.parse_line("WAIT {unknown_key: value}: foo", 5);
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("unrecognized directive parameter"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_timeout_on_send_keys() {
        let mut p = parser();
        let result = p.parse_line("SEND_KEYS {timeout: 5s}: DOWN", 5);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("'timeout' not allowed on SEND_KEYS"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_condition_on_wait() {
        let mut p = parser();
        let result = p.parse_line("WAIT {if_match: \"text\"}: foo", 5);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not allowed on WAIT"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_params_on_assert() {
        let mut p = parser();
        p.parse_line("% timeout: 5s", 5).unwrap();
        let result = p.parse_line("ASSERT: foo", 6);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not allowed on ASSERT"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_params_on_write() {
        let mut p = parser();
        p.parse_line("% timeout: 5s", 5).unwrap();
        let result = p.parse_line("| some text", 6);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not allowed on WRITE"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_inline_overrides_pending() {
        let mut p = parser();
        // Set pending timeout via %
        p.parse_line("% timeout: 3s", 4).unwrap();
        // Inline timeout should override pending
        let dir = p
            .parse_line("WAIT {timeout: 10s}: foo", 5)
            .unwrap()
            .unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Wait {
                pattern: InteractivePattern {
                    expression: "foo".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 6,
                timeout: Some(Duration::from_secs(10)),
            }
        );
    }

    #[test]
    fn test_write_long_form() {
        let mut p = parser();
        let dir = p.parse_line("WRITE: hello world", 4).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Write {
                text: "hello world".to_string(),
                line_number: 5,
            }
        );
    }

    #[test]
    fn test_write_short_form() {
        let mut p = parser();
        let dir = p.parse_line("| hello world", 4).unwrap().unwrap();
        assert_eq!(
            dir,
            InteractiveDirective::Write {
                text: "hello world".to_string(),
                line_number: 5,
            }
        );
    }

    #[test]
    fn test_keyword_not_prefix_matched() {
        // "WAITING" should NOT match "WAIT"
        let mut p = parser();
        let result = p.parse_line("WAITING: foo", 5);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("unrecognized interactive directive"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_parse_send_keys_single_chars() {
        let mut p = parser();
        let dir = p.parse_line("SEND_KEYS: a b c", 4).unwrap().unwrap();
        match dir {
            InteractiveDirective::SendKeys { keys, .. } => {
                assert_eq!(keys, vec!["a", "b", "c"]);
            }
            _ => panic!("expected SendKeys"),
        }
    }

    #[test]
    fn test_parse_send_keys_hex() {
        let mut p = parser();
        let dir = p.parse_line("SEND_KEYS: #0a 0xff", 4).unwrap().unwrap();
        match dir {
            InteractiveDirective::SendKeys { keys, .. } => {
                assert_eq!(keys, vec!["#0a", "0xff"]);
            }
            _ => panic!("expected SendKeys"),
        }
    }

    #[test]
    fn test_error_send_keys_multi_char_word() {
        let mut p = parser();
        let result = p.parse_line("SEND_KEYS: hello ENTER", 4);
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("invalid SEND_KEYS token"),
            "error: {}",
            err_msg
        );
    }

    #[test]
    fn test_error_send_keys_invalid_hex() {
        let mut p = parser();
        let result = p.parse_line("SEND_KEYS: #xyz", 4);
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(err_msg.contains("invalid hex key"), "error: {}", err_msg);
    }
}
