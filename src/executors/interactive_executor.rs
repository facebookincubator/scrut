/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Executor for interactive test cases using PTY sessions.
//!
//! This module provides [`execute_interactive`] which drives a
//! [`scrutty::TtySession`] through a sequence of interactive
//! directives (WAIT, SEND_KEYS, ASSERT) instead of using the standard
//! BashRunner + output diff approach.

use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use regex::Regex;

use super::context::Context as ExecutionContext;
use crate::executors::SHELL_PATH;
use crate::interactive::InteractiveDiff;
use crate::interactive::InteractiveDiffLine;
use crate::interactive::InteractiveDirective;
use crate::interactive::InteractivePattern;
use crate::interactive::PatternKind;
use crate::interactive::SendKeysCondition;
use crate::output::ExitStatus;
use crate::output::Output;
use crate::testcase::TestCase;
use crate::validation::ValidationBody;

#[derive(Debug, Derivative, thiserror::Error)]
#[derivative(PartialEq, Eq)]
pub enum InteractiveError {
    #[error("interactive test failed")]
    DirectivesFailed(InteractiveDiff),
    #[error("internal error in interactive directive: {0}")]
    Internal(String),
}

/// Default timeout WAIT directives
pub static DEFAULT_WAIT_TIMEOUT: LazyLock<Duration> = LazyLock::new(|| Duration::from_mins(1));

/// Delay after each key send, giving the PTY child time to consume the input
/// before the next bytes arrive. Without it, fast-succession keys can be
/// coalesced or dropped by line-buffered readers on the child side.
const KEY_SEND_SETTLE_DELAY: Duration = Duration::from_millis(50);

/// Outcome of a single directive execution, distinguishing "keys intentionally
/// not sent" from success.
enum DirectiveOutcome {
    /// Directive executed (or its keys sent) successfully.
    Passed,
    /// Conditional `SEND_KEYS` whose condition evaluated to false: nothing was
    /// sent, and the directive neither passed nor failed.
    Skipped,
}

/// Execute a single interactive test case using a PTY session.
///
/// Spawns a [`scrutty::TtySession`] with the test case's shell expression,
/// then iterates through the interactive directives (WAIT, SEND_KEYS, ASSERT).
/// Validation happens during execution — each WAIT/ASSERT is verified live.
///
/// # Returns
///
/// An [`Output`] with exit code on success, or an error describing the failed
/// directive (including line number and last terminal output).
pub fn execute_interactive(
    testcase: &TestCase,
    context: &ExecutionContext,
    accumulated_env: &BTreeMap<String, String>,
) -> std::result::Result<Output, InteractiveError> {
    assert!(
        testcase.config.is_interactive(),
        "execute_interactive called on non-interactive test case"
    );

    let shell = context
        .config
        .shell
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new(&*SHELL_PATH));
    let command = shell.to_str().ok_or_else(|| {
        InteractiveError::Internal(format!(
            "shell path is not valid UTF-8: {}",
            shell.display()
        ))
    })?;
    let expression = &testcase.shell_expression;

    // Build environment from testcase config
    let env: Vec<(&str, &str)> = testcase
        .config
        .environment
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let cwd = context.work_directory.to_str().ok_or_else(|| {
        InteractiveError::Internal(format!(
            "work directory is not valid UTF-8: {}",
            context.work_directory.display()
        ))
    })?;

    // Determine per-directive timeout from testcase config or default
    let directive_timeout = testcase
        .config
        .timeout
        .unwrap_or(*DEFAULT_WAIT_TIMEOUT)
        .as_millis() as u64;

    // Extract directives from the validation body
    let directives = match &testcase.body {
        ValidationBody::Interactive(body) => &body.directives,
        _ => {
            return Err(InteractiveError::Internal(
                "execute_interactive called on non-interactive test case".to_string(),
            ));
        }
    };

    // Spawn the PTY session
    let mut session = scrutty::TtySession::spawn(command, &["-c", expression], cwd, env)
        .context("failed to spawn interactive PTY session")
        .map_err(|err| InteractiveError::Internal(err.to_string()))?;

    // Execute each directive sequentially, collecting results
    let mut diff_lines = Vec::with_capacity(directives.len());
    let mut failed = false;

    for directive in directives {
        if failed {
            diff_lines.push(InteractiveDiffLine::Skipped {
                directive: directive.clone(),
            });
            continue;
        }

        let directive = if testcase.config.interpolated == Some(true) && !accumulated_env.is_empty()
        {
            interpolate_directive(directive, accumulated_env)
        } else {
            directive.clone()
        };

        match execute_directive(&session, &directive, directive_timeout) {
            Ok(DirectiveOutcome::Passed) => {
                diff_lines.push(InteractiveDiffLine::Passed { directive });
            }
            Ok(DirectiveOutcome::Skipped) => {
                diff_lines.push(InteractiveDiffLine::Skipped { directive });
            }
            Err(err) => {
                diff_lines.push(InteractiveDiffLine::Failed {
                    directive,
                    error: err.to_string(),
                });
                failed = true;
            }
        }
    }

    if failed {
        let terminal_output = session.get_clean_output();
        return Err(InteractiveError::DirectivesFailed(InteractiveDiff {
            lines: diff_lines,
            terminal_output,
        }));
    }

    // Wait for the process to exit. A timeout here is a failure, not an exit
    // code 0: the session is still running and the test did not complete.
    let exit_code = session.wait_for_exit(directive_timeout).map_err(|err| {
        InteractiveError::Internal(format!(
            "interactive process did not exit within {}: {}",
            humantime::format_duration(Duration::from_millis(directive_timeout)),
            err,
        ))
    })?;

    Ok(Output {
        stdout: vec![].into(),
        stderr: vec![].into(),
        exit_code: ExitStatus::Code(exit_code),
        detached_process: None,
        captured_env: BTreeMap::new(),
        duration: None,
        validation_result: None,
    })
}

/// Execute a single interactive directive against the PTY session.
///
/// Returns whether the directive passed or was skipped (conditional
/// `SEND_KEYS` with a false condition). Failures are returned as errors.
fn execute_directive(
    session: &scrutty::TtySession,
    directive: &InteractiveDirective,
    timeout_ms: u64,
) -> Result<DirectiveOutcome> {
    match directive {
        InteractiveDirective::Wait {
            pattern,
            line_number,
            timeout,
        } => {
            let effective_timeout = timeout.map(|d| d.as_millis() as u64).unwrap_or(timeout_ms);
            wait_for_pattern(session, pattern, effective_timeout).with_context(|| {
                format!(
                    "WAIT at line {} for pattern '{}' failed.\nLast output:\n{}",
                    line_number,
                    pattern,
                    truncate_output(&session.get_clean_output(), 500)
                )
            })?;
        }

        InteractiveDirective::Assert {
            pattern,
            line_number,
        } => {
            assert_pattern(session, pattern).with_context(|| {
                format!(
                    "ASSERT at line {} for pattern '{}' failed.\nLast output:\n{}",
                    line_number,
                    pattern,
                    truncate_output(&session.get_clean_output(), 500)
                )
            })?;
        }

        InteractiveDirective::SendKeys {
            keys,
            condition,
            line_number,
        } => {
            // Evaluate condition if present; a false condition means the keys
            // are intentionally not sent (skipped, not passed).
            if let Some(condition) = condition {
                if !evaluate_condition(session, condition)? {
                    return Ok(DirectiveOutcome::Skipped);
                }
            }

            for key in keys {
                let resolved = resolve_key(key)
                    .with_context(|| format!("SEND_KEYS at line {}", line_number))?;
                session
                    .write_bytes(&resolved)
                    .context("failed to send key")?;
                std::thread::sleep(KEY_SEND_SETTLE_DELAY);
            }
        }

        InteractiveDirective::Write {
            text,
            line_number: _,
        } => {
            session
                .write(&format!("{}\n", text))
                .context("failed to write text to STDIN")?;
        }
    }
    Ok(DirectiveOutcome::Passed)
}

/// Resolve a SEND_KEYS token to its byte sequence.
///
/// Valid tokens: single char, hex (#xx / 0xxx), or named key (ENTER, DOWN, etc.).
/// Returns an error for unrecognized multi-char tokens.
fn resolve_key(token: &str) -> Result<Vec<u8>> {
    // Single character: pass through as UTF-8 bytes
    if token.chars().count() == 1 {
        return Ok(token.as_bytes().to_vec());
    }

    // Hex byte: #xx or 0xxx. Sent as a single raw byte: routing through
    // `char` would re-encode 0x80-0xFF as two-byte UTF-8.
    if let Some(byte) = parse_hex_byte(token) {
        return Ok(vec![byte]);
    }

    // Named key: resolved by scrutty
    let resolved = scrutty::session::key_name_to_sequence(token);
    if resolved != token {
        return Ok(resolved.as_bytes().to_vec());
    }

    // Unrecognized
    bail!(
        "unrecognized key name {:?}; valid keys: single chars (a, b), \
         named keys (ENTER, DOWN, CTRL+C), or hex (#0a, 0xff)",
        token
    )
}

fn parse_hex_byte(token: &str) -> Option<u8> {
    let hex = token
        .strip_prefix('#')
        .or_else(|| token.strip_prefix("0x"))?;
    u8::from_str_radix(hex, 16).ok()
}

/// Wait for PTY output to match the given pattern
fn wait_for_pattern(
    session: &scrutty::TtySession,
    pattern: &InteractivePattern,
    timeout_ms: u64,
) -> Result<()> {
    match pattern.kind {
        PatternKind::Regex => {
            let regex = Regex::new(&pattern.expression).context("invalid regex pattern in WAIT")?;
            session.wait_for_regex(&regex, timeout_ms, true)?;
        }
        PatternKind::Glob => {
            let regex_pattern = glob_to_regex(&pattern.expression);
            let regex =
                Regex::new(&regex_pattern).context("failed to convert glob to regex for WAIT")?;
            session.wait_for_regex(&regex, timeout_ms, true)?;
        }
        PatternKind::Equal => {
            session.wait_for_output(&pattern.expression, timeout_ms, true)?;
        }
    }
    Ok(())
}

/// Assert that current PTY output matches the given pattern (non-blocking)
fn assert_pattern(session: &scrutty::TtySession, pattern: &InteractivePattern) -> Result<()> {
    let output = session.get_clean_output();
    let matches = pattern_matches(&output, pattern)?;

    if !matches {
        bail!(
            "assertion failed: expected output to match '{}' ({})",
            pattern.expression,
            pattern.kind,
        );
    }
    Ok(())
}

/// Evaluate a SEND_KEYS condition against current PTY output
fn evaluate_condition(
    session: &scrutty::TtySession,
    condition: &SendKeysCondition,
) -> Result<bool> {
    let output = session.get_clean_output();
    match condition {
        SendKeysCondition::IfMatch(pattern) => pattern_matches(&output, pattern),
        SendKeysCondition::UnlessMatch(pattern) => Ok(!pattern_matches(&output, pattern)?),
    }
}

/// Check whether output matches a pattern.
///
/// Glob matching is intentionally a substring (unanchored) match, mirroring
/// the streaming `WAIT` semantics: `ASSERT` checks the accumulated output with
/// the same matcher the `WAIT` loop applies to the stream.
fn pattern_matches(output: &str, pattern: &InteractivePattern) -> Result<bool> {
    match pattern.kind {
        PatternKind::Regex => {
            let regex = Regex::new(&pattern.expression)?;
            Ok(regex.is_match(output))
        }
        PatternKind::Glob => {
            let regex_pattern = glob_to_regex(&pattern.expression);
            let regex = Regex::new(&regex_pattern)?;
            Ok(regex.is_match(output))
        }
        PatternKind::Equal => Ok(output.contains(&pattern.expression)),
    }
}

/// Convert a glob pattern to a regex pattern.
///
/// Supports `*` (any sequence) and `?` (any single char). All other regex
/// metacharacters are escaped.
fn glob_to_regex(glob: &str) -> String {
    let mut regex = String::from("(?s)");
    for ch in glob.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '%' | '[' | ']' | '{' | '}' | '\\' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    regex
}

/// Interpolate `$VAR` references in a directive's patterns using the given env map.
fn interpolate_directive(
    directive: &InteractiveDirective,
    env: &BTreeMap<String, String>,
) -> InteractiveDirective {
    let interpolate_pattern = |p: &InteractivePattern| InteractivePattern {
        expression: crate::interpolation::interpolate_str(&p.expression, env),
        kind: p.kind,
    };
    let interpolate_condition = |c: &SendKeysCondition| match c {
        SendKeysCondition::IfMatch(p) => SendKeysCondition::IfMatch(interpolate_pattern(p)),
        SendKeysCondition::UnlessMatch(p) => SendKeysCondition::UnlessMatch(interpolate_pattern(p)),
    };

    match directive {
        InteractiveDirective::Wait {
            pattern,
            line_number,
            timeout,
        } => InteractiveDirective::Wait {
            pattern: interpolate_pattern(pattern),
            line_number: *line_number,
            timeout: *timeout,
        },
        InteractiveDirective::Assert {
            pattern,
            line_number,
        } => InteractiveDirective::Assert {
            pattern: interpolate_pattern(pattern),
            line_number: *line_number,
        },
        InteractiveDirective::Write { text, line_number } => InteractiveDirective::Write {
            text: crate::interpolation::interpolate_str(text, env),
            line_number: *line_number,
        },
        InteractiveDirective::SendKeys {
            keys,
            condition,
            line_number,
        } => InteractiveDirective::SendKeys {
            keys: keys.clone(),
            condition: condition.as_ref().map(interpolate_condition),
            line_number: *line_number,
        },
    }
}

/// Truncate output string to at most `max_len` bytes, keeping the end.
///
/// The cut is moved back to the previous UTF-8 character boundary so slicing
/// never panics on multi-byte terminal output.
fn truncate_output(output: &str, max_len: usize) -> &str {
    if output.len() <= max_len {
        return output;
    }
    let mut start = output.len() - max_len;
    while !output.is_char_boundary(start) {
        start += 1;
    }
    &output[start..]
}

#[cfg(all(test, unix))]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::PathBuf;

    use super::InteractiveError;
    use super::execute_interactive;
    use crate::config::DocumentConfig;
    use crate::config::TestCaseConfig;
    use crate::config::TestMode;
    use crate::executors::context::Context;
    use crate::testcase::TestCase;
    use crate::validation::InteractiveBody;
    use crate::validation::ValidationBody;

    fn non_utf8_path() -> PathBuf {
        PathBuf::from(OsStr::from_bytes(b"/tmp/scrut-\xff"))
    }

    fn interactive_testcase() -> TestCase {
        TestCase {
            shell_expression: "true".to_string(),
            body: ValidationBody::Interactive(InteractiveBody { directives: vec![] }),
            config: TestCaseConfig {
                mode: Some(TestMode::Interactive),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // Built directly rather than via `Context::new_for_test`, so no testing
    // directories are created (or cleaned up) for a path that cannot exist.
    fn context(work_directory: PathBuf, config: DocumentConfig) -> Context {
        Context {
            work_directory,
            temp_directory: PathBuf::from("/nonexistent"),
            file: PathBuf::from("test.md"),
            config,
        }
    }

    fn expect_internal_error(context: &Context, needle: &str) {
        let result = execute_interactive(&interactive_testcase(), context, &BTreeMap::new());
        match result {
            Err(InteractiveError::Internal(msg)) => assert!(
                msg.contains(needle),
                "error should mention {needle:?}, got: {msg}"
            ),
            other => panic!("expected an internal error, not a fallback: {other:?}"),
        }
    }

    #[test]
    fn test_non_utf8_work_directory_is_an_error() {
        let context = context(non_utf8_path(), DocumentConfig::default());
        expect_internal_error(&context, "work directory is not valid UTF-8");
    }

    #[test]
    fn test_non_utf8_shell_is_an_error() {
        let config = DocumentConfig {
            shell: Some(non_utf8_path()),
            ..Default::default()
        };
        let context = context(PathBuf::from("/tmp"), config);
        expect_internal_error(&context, "shell path is not valid UTF-8");
    }
}
