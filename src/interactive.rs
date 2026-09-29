/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Types for the interactive validation mode.
//!
//! Interactive mode allows testing interactive CLI tools (menus, prompts, etc.)
//! by driving a PTY session with directives like WAIT, SEND_KEYS, and ASSERT,
//! rather than comparing static output expectations.
//!
//! ## Syntax
//!
//! ### Long form
//!
//! ```text
//! WAIT: Select a flavor
//! WAIT {timeout: 5s}: Select a flavor
//! SEND_KEYS: DOWN DOWN DOWN ENTER
//! ASSERT: .*something.* (regex)
//! SEND_KEYS {unless_match: "some text"}: DOWN
//! ```
//!
//! ### Short form
//!
//! ```text
//! @ Select a flavor
//! ^ DOWN DOWN DOWN ENTER
//! ! .*something.* (regex)
//! % timeout: 5s
//! @ Select a flavor
//! % unless_match: "some text"
//! ^ DOWN
//! ```

use std::fmt;
use std::time::Duration;

use serde::Serialize;

/// Result of executing a single interactive directive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InteractiveDiffLine {
    /// Directive executed successfully
    Passed { directive: InteractiveDirective },
    /// Directive failed with an error message
    Failed {
        directive: InteractiveDirective,
        error: String,
    },
    /// Directive was not executed (because a prior directive failed)
    Skipped { directive: InteractiveDirective },
}

impl Serialize for InteractiveDiffLine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            Self::Passed { directive } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("status", "passed")?;
                map.serialize_entry("directive", &directive.to_string())?;
                map.serialize_entry("line_number", &directive.line_number())?;
                map.end()
            }
            Self::Failed { directive, error } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("status", "failed")?;
                map.serialize_entry("directive", &directive.to_string())?;
                map.serialize_entry("line_number", &directive.line_number())?;
                map.serialize_entry("error", error)?;
                map.end()
            }
            Self::Skipped { directive } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("status", "skipped")?;
                map.serialize_entry("directive", &directive.to_string())?;
                map.serialize_entry("line_number", &directive.line_number())?;
                map.end()
            }
        }
    }
}

impl InteractiveDiffLine {
    /// Returns the directive's line number in the source file.
    pub fn line_number(&self) -> usize {
        match self {
            Self::Passed { directive }
            | Self::Failed { directive, .. }
            | Self::Skipped { directive } => directive.line_number(),
        }
    }
}

/// Structured result of an interactive test execution, carrying per-directive
/// pass/fail/skip status. Analogous to `Diff` for output-mode tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveDiff {
    pub lines: Vec<InteractiveDiffLine>,
    /// Terminal output at the time of failure (for debugging context)
    pub terminal_output: String,
}

impl InteractiveDiff {
    /// Returns true if any directive failed.
    pub fn has_failures(&self) -> bool {
        self.lines
            .iter()
            .any(|line| matches!(line, InteractiveDiffLine::Failed { .. }))
    }
}

impl Serialize for InteractiveDiff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("lines", &self.lines)?;
        map.serialize_entry("terminal_output", &self.terminal_output)?;
        map.end()
    }
}

/// How an [`InteractivePattern`] is matched against terminal output.
///
/// Mirrors the rule-kind vocabulary of output expectations (`equal`, `glob`,
/// `regex`), including their short aliases (`eq`, `gl`, `re`) when parsing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PatternKind {
    /// Literal text match (contains-match against terminal output).
    #[default]
    Equal,
    /// Wildcard match (`*` any sequence, `?` any single char).
    Glob,
    /// Regular expression match.
    Regex,
}

impl PatternKind {
    /// Canonical name, as rendered by [`fmt::Display`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equal => "equal",
            Self::Glob => "glob",
            Self::Regex => "regex",
        }
    }
}

impl std::str::FromStr for PatternKind {
    type Err = anyhow::Error;

    /// Parse a rule-kind name, accepting the short aliases of the expectation
    /// grammar (`eq`, `gl`, `re`) alongside the canonical names.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "equal" | "eq" => Ok(Self::Equal),
            "glob" | "gl" => Ok(Self::Glob),
            "regex" | "re" => Ok(Self::Regex),
            _ => Err(anyhow::anyhow!("unknown pattern kind {:?}", s)),
        }
    }
}

impl fmt::Display for PatternKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for PatternKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// A match pattern used in WAIT, ASSERT, and conditional SEND_KEYS.
///
/// Reuses the same `(kind)` suffix syntax as output expectations:
/// - No suffix: equal (literal text match)
/// - `(glob)`: wildcard match
/// - `(regex)`: regular expression match
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractivePattern {
    /// The pattern expression (e.g., "Select a flavor", ".*flavor.*")
    pub expression: String,
    /// How the expression is matched.
    pub kind: PatternKind,
}

impl fmt::Display for InteractivePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kind == PatternKind::Equal {
            write!(f, "{}", self.expression)
        } else {
            write!(f, "{} ({})", self.expression, self.kind)
        }
    }
}

/// Condition for conditional SEND_KEYS directives.
///
/// Matchers use `"string"` for contains-match and `/regex/` for regex match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendKeysCondition {
    /// Only send keys if the current output matches the pattern
    IfMatch(InteractivePattern),
    /// Only send keys unless the current output matches the pattern
    UnlessMatch(InteractivePattern),
}

impl fmt::Display for SendKeysCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IfMatch(pattern) => write!(f, "if_match: {}", pattern),
            Self::UnlessMatch(pattern) => write!(f, "unless_match: {}", pattern),
        }
    }
}

/// A single directive in an interactive test body.
///
/// Directives are executed sequentially against a PTY session during test execution.
/// Unlike output expectations, interactive directives perform actions and validations
/// in real-time as the process runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InteractiveDirective {
    /// Wait for output matching the pattern (with timeout).
    ///
    /// Long form: `WAIT: <pattern>` or `WAIT {timeout: 5s}: <pattern>`
    /// Short form: `@ <pattern>`
    ///
    /// Blocks until the PTY output contains text matching the pattern, or times out.
    /// An optional per-directive timeout overrides the testcase-level timeout.
    Wait {
        pattern: InteractivePattern,
        line_number: usize,
        timeout: Option<Duration>,
    },

    /// Send named keys to the PTY session.
    ///
    /// Long form: `SEND_KEYS: KEY1 KEY2` or `SEND_KEYS {condition}: KEY1 KEY2`
    /// Short form: `^ KEY1 KEY2`
    ///
    /// Each token must be one of:
    /// - A **named key**: ENTER, DOWN, CTRL+C, TAB, etc.
    /// - A **single character**: a, b, 1, !, etc.
    /// - A **hex byte**: #0a, 0xff (1-2 hex digits)
    ///
    /// Multi-character words (e.g. `hello`) are NOT valid tokens.
    /// To type a word, either use `WRITE: hello` (appends newline) or
    /// spell out characters: `SEND_KEYS: h e l l o ENTER`.
    ///
    /// ## Named keys
    ///
    /// **Arrows:** UP, DOWN, LEFT, RIGHT
    /// **Common:** ENTER, SPACE, TAB, ESCAPE (ESC), BACKSPACE, DELETE (DEL)
    /// **Control:** CTRL+A .. CTRL+Z (e.g. CTRL+C, CTRL+D)
    /// **Function:** F1 .. F12
    /// **Navigation:** HOME, END, PAGEUP (PAGE_UP), PAGEDOWN (PAGE_DOWN), INSERT (INS)
    ///
    /// An optional condition controls whether the keys are actually sent.
    SendKeys {
        keys: Vec<String>,
        condition: Option<SendKeysCondition>,
        line_number: usize,
    },

    /// Assert that current output matches pattern (fail immediately if not).
    ///
    /// Long form: `ASSERT: <pattern>`
    /// Short form: `! <pattern>`
    ///
    /// Unlike WAIT, does not block — checks the current accumulated output immediately.
    Assert {
        pattern: InteractivePattern,
        line_number: usize,
    },

    /// Write text to the STDIN of the running process, terminated with a newline.
    ///
    /// Long form: `WRITE: <text>`
    /// Short form: `| <text>`
    ///
    /// Sends the given text followed by `\n` directly to the process's standard input.
    Write { text: String, line_number: usize },
}

impl InteractiveDirective {
    /// Returns the source line number of this directive.
    pub fn line_number(&self) -> usize {
        match self {
            Self::Wait { line_number, .. }
            | Self::SendKeys { line_number, .. }
            | Self::Assert { line_number, .. }
            | Self::Write { line_number, .. } => *line_number,
        }
    }
}

impl fmt::Display for InteractiveDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wait {
                pattern, timeout, ..
            } => {
                if let Some(timeout) = timeout {
                    // Human duration (`5s`, not `5000ms`) so the rendered form
                    // matches the documented syntax and round-trips through the
                    // parser (which accepts humantime durations).
                    write!(
                        f,
                        "WAIT {{timeout: {}}}: {}",
                        humantime::format_duration(*timeout),
                        pattern
                    )
                } else {
                    write!(f, "WAIT: {}", pattern)
                }
            }
            Self::SendKeys {
                keys, condition, ..
            } => {
                let content = keys.join(" ");
                if let Some(condition) = condition {
                    write!(f, "SEND_KEYS {{{}}}: {}", condition, content)
                } else {
                    write!(f, "SEND_KEYS: {}", content)
                }
            }
            Self::Assert { pattern, .. } => {
                write!(f, "ASSERT: {}", pattern)
            }
            Self::Write { text, .. } => {
                write!(f, "WRITE: {}", text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_pattern_display_equal() {
        let pattern = InteractivePattern {
            expression: "Select a flavor".to_string(),
            kind: PatternKind::Equal,
        };
        assert_eq!(format!("{}", pattern), "Select a flavor");
    }

    #[test]
    fn test_interactive_pattern_display_regex() {
        let pattern = InteractivePattern {
            expression: ".*flavor.*".to_string(),
            kind: PatternKind::Regex,
        };
        assert_eq!(format!("{}", pattern), ".*flavor.* (regex)");
    }

    #[test]
    fn test_pattern_kind_parses_aliases() {
        for (alias, expected) in [
            ("equal", PatternKind::Equal),
            ("eq", PatternKind::Equal),
            ("glob", PatternKind::Glob),
            ("gl", PatternKind::Glob),
            ("regex", PatternKind::Regex),
            ("re", PatternKind::Regex),
        ] {
            let parsed: PatternKind = alias.parse().expect("parse kind");
            assert_eq!(parsed, expected);
            // Aliases normalize to the canonical name on render, so an `eq`
            // pattern never renders as `... (eq)`.
            assert_eq!(expected.as_str(), expected.to_string());
        }
        assert!("fuzzy".parse::<PatternKind>().is_err());
    }

    #[test]
    fn test_send_keys_condition_display() {
        let cond = SendKeysCondition::IfMatch(InteractivePattern {
            expression: "text".to_string(),
            kind: PatternKind::Equal,
        });
        assert_eq!(format!("{}", cond), "if_match: text");

        let cond = SendKeysCondition::UnlessMatch(InteractivePattern {
            expression: ".*pat.*".to_string(),
            kind: PatternKind::Regex,
        });
        assert_eq!(format!("{}", cond), "unless_match: .*pat.* (regex)");
    }

    #[test]
    fn test_directive_display_wait() {
        let dir = InteractiveDirective::Wait {
            pattern: InteractivePattern {
                expression: "Select a flavor".to_string(),
                kind: PatternKind::Equal,
            },
            line_number: 5,
            timeout: None,
        };
        assert_eq!(format!("{}", dir), "WAIT: Select a flavor");
    }

    #[test]
    fn test_directive_display_wait_timeout_round_trips() {
        let dir = InteractiveDirective::Wait {
            pattern: InteractivePattern {
                expression: "Select a flavor".to_string(),
                kind: PatternKind::Equal,
            },
            line_number: 5,
            timeout: Some(Duration::from_secs(5)),
        };
        let rendered = format!("{}", dir);
        assert_eq!(rendered, "WAIT {timeout: 5s}: Select a flavor");
        // The rendered timeout parses back via humantime, matching the
        // parser's accepted duration syntax.
        let timeout_str = rendered
            .strip_prefix("WAIT {timeout: ")
            .and_then(|s| s.strip_suffix("}: Select a flavor"))
            .expect("rendered WAIT shape");
        assert_eq!(
            humantime::parse_duration(timeout_str).expect("parse timeout"),
            Duration::from_secs(5)
        );
    }

    #[test]
    fn test_diff_line_serialization_includes_line_number() {
        let line = InteractiveDiffLine::Passed {
            directive: InteractiveDirective::Assert {
                pattern: InteractivePattern {
                    expression: "done".to_string(),
                    kind: PatternKind::Equal,
                },
                line_number: 8,
            },
        };
        let value = serde_json::to_value(&line).expect("serialize diff line");
        assert_eq!(value["status"], "passed");
        assert_eq!(value["line_number"], 8);
    }

    #[test]
    fn test_directive_display_send_keys() {
        let dir = InteractiveDirective::SendKeys {
            keys: vec!["DOWN".to_string(), "ENTER".to_string()],
            condition: None,
            line_number: 6,
        };
        assert_eq!(format!("{}", dir), "SEND_KEYS: DOWN ENTER");
    }

    #[test]
    fn test_directive_display_send_keys_with_condition() {
        let dir = InteractiveDirective::SendKeys {
            keys: vec!["DOWN".to_string()],
            condition: Some(SendKeysCondition::UnlessMatch(InteractivePattern {
                expression: "flavor".to_string(),
                kind: PatternKind::Equal,
            })),
            line_number: 7,
        };
        assert_eq!(format!("{}", dir), "SEND_KEYS {unless_match: flavor}: DOWN");
    }

    #[test]
    fn test_directive_display_assert() {
        let dir = InteractiveDirective::Assert {
            pattern: InteractivePattern {
                expression: ".*something.*".to_string(),
                kind: PatternKind::Regex,
            },
            line_number: 8,
        };
        assert_eq!(format!("{}", dir), "ASSERT: .*something.* (regex)");
    }
}
