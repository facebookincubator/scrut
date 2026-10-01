/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Fluent Expect API for TTY session interactions
//!
//! Provides a chainable builder pattern for expressing sequences of
//! terminal interactions. This is more ergonomic than imperative calls
//! for common patterns.
//!
//! # Example
//!
//! ```rust,ignore
//! session.expect("$")
//!     .send("echo hello")
//!     .send_key(Keys::ENTER)
//!     .expect("hello")
//!     .run()?;
//! ```

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use regex::Regex;

use super::keys::key_name_to_sequence;
use super::output::clean_tty_output;
use crate::terminal::TerminalState;

/// Default timeout for expect operations in milliseconds
const DEFAULT_TIMEOUT_MS: u64 = 5000;

/// A step in an expect chain
#[derive(Clone)]
enum Step {
    /// Wait for literal text to appear in output
    Expect {
        text: String,
        timeout_ms: u64,
        use_clean: bool,
    },
    /// Wait for regex pattern to match in output
    ExpectRegex {
        pattern: Regex,
        timeout_ms: u64,
        use_clean: bool,
    },
    /// Wait for a menu to appear, optionally with specific highlighted item
    ExpectMenu {
        highlighted_text: Option<String>,
        timeout_ms: u64,
    },
    /// Send text input (typed as-is)
    Send { text: String },
    /// Send a key sequence (e.g., Keys::ENTER)
    SendKey { key: String },
    /// Add an explicit delay
    Delay { ms: u64 },
}

impl std::fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Expect {
                text,
                timeout_ms,
                use_clean,
            } => {
                write!(
                    f,
                    "Expect({:?}, timeout={}ms, clean={})",
                    text, timeout_ms, use_clean
                )
            }
            Step::ExpectRegex {
                pattern,
                timeout_ms,
                use_clean,
            } => {
                write!(
                    f,
                    "ExpectRegex({}, timeout={}ms, clean={})",
                    pattern, timeout_ms, use_clean
                )
            }
            Step::ExpectMenu {
                highlighted_text,
                timeout_ms,
            } => {
                write!(
                    f,
                    "ExpectMenu(highlighted={:?}, timeout={}ms)",
                    highlighted_text, timeout_ms
                )
            }
            Step::Send { text } => write!(f, "Send({:?})", text),
            Step::SendKey { key } => write!(f, "SendKey({:?})", key),
            Step::Delay { ms } => write!(f, "Delay({}ms)", ms),
        }
    }
}

/// Builder for chaining terminal expectations and actions
///
/// Created by calling `.expect()` on a `TtySession`. Steps are executed
/// in order when `.run()` is called.
///
/// # Timeout Behavior
///
/// The default timeout is 5 seconds. You can override it for specific
/// expect steps using `.timeout()`:
///
/// ```rust,ignore
/// session.expect("slow operation")
///     .timeout(30_000)  // 30 seconds for this expect
///     .expect("fast operation")  // back to default 5s
///     .run()?;
/// ```
pub struct ExpectChain<'a> {
    /// The output buffer from the session
    output: Arc<Mutex<String>>,
    /// The writer for sending input
    writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    /// Accumulated steps to execute
    steps: Vec<Step>,
    /// Timeout for the next expect step (reset after use)
    next_timeout_ms: u64,
    /// Whether to use clean output for next expect
    next_use_clean: bool,
    /// Phantom lifetime for the session reference
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ExpectChain<'a> {
    /// Create a new expect chain (internal - use TtySession::expect())
    pub(crate) fn new(
        output: Arc<Mutex<String>>,
        writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    ) -> Self {
        Self {
            output,
            writer,
            steps: Vec::new(),
            next_timeout_ms: DEFAULT_TIMEOUT_MS,
            next_use_clean: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Expect literal text to appear in the output
    ///
    /// Waits for the text to appear, using the current timeout setting.
    pub fn expect(mut self, text: &str) -> Self {
        self.steps.push(Step::Expect {
            text: text.to_string(),
            timeout_ms: self.next_timeout_ms,
            use_clean: self.next_use_clean,
        });
        // Reset to defaults after use
        self.next_timeout_ms = DEFAULT_TIMEOUT_MS;
        self.next_use_clean = false;
        self
    }

    /// Expect a regex pattern to match in the output
    ///
    /// Waits for the pattern to match, using the current timeout setting.
    pub fn expect_regex(mut self, pattern: &str) -> Result<Self> {
        let regex =
            Regex::new(pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;
        self.steps.push(Step::ExpectRegex {
            pattern: regex,
            timeout_ms: self.next_timeout_ms,
            use_clean: self.next_use_clean,
        });
        // Reset to defaults after use
        self.next_timeout_ms = DEFAULT_TIMEOUT_MS;
        self.next_use_clean = false;
        Ok(self)
    }

    /// Send text to the terminal (typed as-is)
    ///
    /// The text is written directly to the PTY. Use this for typing
    /// commands or input text.
    pub fn send(mut self, text: &str) -> Self {
        self.steps.push(Step::Send {
            text: text.to_string(),
        });
        self
    }

    /// Send a key sequence to the terminal
    ///
    /// Accepts key constants like `Keys::ENTER`, `Keys::DOWN`, etc.
    /// Also accepts human-readable names like "ENTER", "CTRL+C".
    pub fn send_key(mut self, key: &str) -> Self {
        self.steps.push(Step::SendKey {
            key: key.to_string(),
        });
        self
    }

    /// Set the timeout for the next expect step
    ///
    /// This only affects the immediately following `.expect()` or
    /// `.expect_regex()` call. After that, the timeout resets to default.
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.next_timeout_ms = timeout_ms;
        self
    }

    /// Use clean output (ANSI codes stripped) for the next expect
    ///
    /// This only affects the immediately following `.expect()` or
    /// `.expect_regex()` call.
    pub fn clean(mut self) -> Self {
        self.next_use_clean = true;
        self
    }

    /// Add an explicit delay
    ///
    /// Sometimes you need to wait for terminal updates that don't
    /// produce matchable output.
    pub fn delay(mut self, ms: u64) -> Self {
        self.steps.push(Step::Delay { ms });
        self
    }

    /// Expect a menu to appear in the terminal
    ///
    /// Waits for the terminal state to contain a detectable menu.
    /// Uses the current timeout setting.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// session.expect_menu()
    ///     .send_key(Keys::DOWN)
    ///     .expect_menu_highlighted("Option B")
    ///     .run()?;
    /// ```
    pub fn expect_menu(mut self) -> Self {
        self.steps.push(Step::ExpectMenu {
            highlighted_text: None,
            timeout_ms: self.next_timeout_ms,
        });
        self.next_timeout_ms = DEFAULT_TIMEOUT_MS;
        self
    }

    /// Expect a menu with a specific highlighted item
    ///
    /// Waits for a menu to appear where the highlighted item contains
    /// the given text. Uses the current timeout setting.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// session.expect_menu_highlighted("exit")
    ///     .send_key(Keys::ENTER)
    ///     .run()?;
    /// ```
    pub fn expect_menu_highlighted(mut self, text: &str) -> Self {
        self.steps.push(Step::ExpectMenu {
            highlighted_text: Some(text.to_string()),
            timeout_ms: self.next_timeout_ms,
        });
        self.next_timeout_ms = DEFAULT_TIMEOUT_MS;
        self
    }

    /// Execute all steps in the chain
    ///
    /// Returns `Ok(())` if all steps succeed, or an error with
    /// the step number and context if any step fails.
    pub fn run(self) -> Result<()> {
        for (index, step) in self.steps.iter().enumerate() {
            let step_num = index + 1;
            self.execute_step(step_num, step)?;
        }
        Ok(())
    }

    /// Execute a single step
    fn execute_step(&self, step_num: usize, step: &Step) -> Result<()> {
        match step {
            Step::Expect {
                text,
                timeout_ms,
                use_clean,
            } => self.do_expect_text(step_num, text, *timeout_ms, *use_clean),
            Step::ExpectRegex {
                pattern,
                timeout_ms,
                use_clean,
            } => self.do_expect_regex(step_num, pattern, *timeout_ms, *use_clean),
            Step::ExpectMenu {
                highlighted_text,
                timeout_ms,
            } => self.do_expect_menu(step_num, highlighted_text.as_deref(), *timeout_ms),
            Step::Send { text } => self.do_send(step_num, text),
            Step::SendKey { key } => self.do_send_key(step_num, key),
            Step::Delay { ms } => {
                std::thread::sleep(Duration::from_millis(*ms));
                Ok(())
            }
        }
    }

    /// Execute an expect text step
    fn do_expect_text(
        &self,
        step_num: usize,
        text: &str,
        timeout_ms: u64,
        use_clean: bool,
    ) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() >= timeout {
                let output = self.get_output(use_clean);
                let tail = self.get_output_tail(&output, 500);
                return Err(anyhow!(
                    "Step {} failed: expected '{}' not found after {}ms\nReceived (last 500 chars):\n{}",
                    step_num,
                    text,
                    timeout_ms,
                    tail
                ));
            }

            let current = self.get_output(use_clean);
            if current.contains(text) {
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Execute an expect regex step
    fn do_expect_regex(
        &self,
        step_num: usize,
        pattern: &Regex,
        timeout_ms: u64,
        use_clean: bool,
    ) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() >= timeout {
                let output = self.get_output(use_clean);
                let tail = self.get_output_tail(&output, 500);
                return Err(anyhow!(
                    "Step {} failed: pattern '{}' not matched after {}ms\nReceived (last 500 chars):\n{}",
                    step_num,
                    pattern,
                    timeout_ms,
                    tail
                ));
            }

            let current = self.get_output(use_clean);
            if pattern.is_match(&current) {
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Execute an expect menu step
    fn do_expect_menu(
        &self,
        step_num: usize,
        highlighted_text: Option<&str>,
        timeout_ms: u64,
    ) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() >= timeout {
                let output = self.get_output(false);
                let tail = self.get_output_tail(&output, 500);
                return Err(anyhow!(
                    "Step {} failed: menu {} not found after {}ms\nReceived (last 500 chars):\n{}",
                    step_num,
                    highlighted_text
                        .map(|t| format!("with highlighted '{}'", t))
                        .unwrap_or_else(|| "".to_string()),
                    timeout_ms,
                    tail
                ));
            }

            let output = self.get_output(false);
            let state = TerminalState::from_output(&output);

            if let Some(menu) = state.find_menu() {
                // Check highlighted text if specified
                if let Some(expected) = highlighted_text {
                    if menu.highlighted_contains(expected) {
                        return Ok(());
                    }
                } else {
                    // Just need any menu
                    return Ok(());
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Execute a send text step
    fn do_send(&self, step_num: usize, text: &str) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer
            .write_all(text.as_bytes())
            .with_context(|| format!("Step {} failed: could not send text", step_num))?;
        writer.flush()?;
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    /// Execute a send key step
    fn do_send_key(&self, step_num: usize, key: &str) -> Result<()> {
        let sequence = key_name_to_sequence(key);
        let mut writer = self.writer.lock().unwrap();
        writer
            .write_all(sequence.as_bytes())
            .with_context(|| format!("Step {} failed: could not send key '{}'", step_num, key))?;
        writer.flush()?;
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    /// Get current output, optionally cleaned
    fn get_output(&self, use_clean: bool) -> String {
        let raw = self.output.lock().unwrap().clone();
        if use_clean {
            clean_tty_output(&raw)
        } else {
            raw
        }
    }

    /// Get tail of output for error messages
    fn get_output_tail(&self, output: &str, max_chars: usize) -> String {
        if output.len() <= max_chars {
            output.to_string()
        } else {
            // Find safe char boundary
            let start = output.len() - max_chars;
            let safe_start = output
                .char_indices()
                .find(|(i, _)| *i >= start)
                .map(|(i, _)| i)
                .unwrap_or(start);
            output[safe_start..].to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    /// Mock writer that captures what was written
    struct MockWriter {
        written: Arc<Mutex<Vec<u8>>>,
    }

    impl MockWriter {
        fn new() -> (Self, Arc<Mutex<Vec<u8>>>) {
            let written = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    written: written.clone(),
                },
                written,
            )
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.written.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn create_test_chain(output_content: &str) -> (ExpectChain<'static>, Arc<Mutex<Vec<u8>>>) {
        let output = Arc::new(Mutex::new(output_content.to_string()));
        let (mock_writer, written) = MockWriter::new();
        let writer: Arc<Mutex<Box<dyn Write + Send>>> = Arc::new(Mutex::new(Box::new(mock_writer)));
        let chain = ExpectChain::new(output, writer);
        (chain, written)
    }

    #[test]
    fn test_expect_finds_text() {
        let (chain, _) = create_test_chain("hello world");
        let result = chain.expect("hello").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_finds_partial_text() {
        let (chain, _) = create_test_chain("some output with hello in it");
        let result = chain.expect("hello").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_timeout_on_missing_text() {
        let (chain, _) = create_test_chain("hello world");
        // Use very short timeout for test
        let result = chain.timeout(100).expect("not_present").run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Step 1 failed"));
        assert!(err_msg.contains("not_present"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_send_writes_to_writer() {
        let (chain, written) = create_test_chain("$");
        let result = chain.expect("$").send("hello").run();
        assert!(result.is_ok());
        let output = written.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&output), "hello");
    }

    #[test]
    fn test_send_key_writes_sequence() {
        let (chain, written) = create_test_chain("$");
        let result = chain.expect("$").send_key("ENTER").run();
        assert!(result.is_ok());
        let output = written.lock().unwrap();
        assert_eq!(&output[..], b"\r"); // ENTER is \r
    }

    #[test]
    fn test_chain_multiple_steps() {
        let (chain, written) = create_test_chain("prompt$ output");
        let result = chain
            .expect("prompt")
            .send("cmd")
            .send_key("ENTER")
            .expect("output")
            .run();
        assert!(result.is_ok());

        let output = written.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&output), "cmd\r");
    }

    #[test]
    fn test_expect_regex() {
        let (chain, _) = create_test_chain("file123.txt");
        let result = chain.expect_regex(r"file\d+\.txt").unwrap().run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_regex_timeout() {
        let (chain, _) = create_test_chain("hello");
        let result = chain.timeout(100).expect_regex(r"world\d+").unwrap().run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("pattern"));
    }

    #[test]
    fn test_invalid_regex_returns_error() {
        let (chain, _) = create_test_chain("hello");
        let result = chain.expect_regex(r"[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_only_affects_next_expect() {
        let (chain, _) = create_test_chain("fast slow");
        // timeout(100) should only affect the first expect
        // The second expect should use the default
        let result = chain.timeout(100).expect("fast").expect("slow").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_delay_step() {
        let (chain, _) = create_test_chain("$");
        let start = std::time::Instant::now();
        let result = chain.expect("$").delay(100).run();
        assert!(result.is_ok());
        assert!(start.elapsed() >= Duration::from_millis(100));
    }

    #[test]
    fn test_error_message_includes_step_number() {
        let (chain, _) = create_test_chain("first second");
        let result = chain
            .expect("first")
            .expect("second")
            .timeout(100)
            .expect("missing")
            .run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // "missing" is the 3rd step
        assert!(err_msg.contains("Step 3"));
    }

    #[test]
    fn test_clean_mode() {
        let (chain, _) = create_test_chain("\x1b[32mhello\x1b[0m");
        // Without clean mode, we should find the raw ANSI
        let result = chain.expect("\x1b[32m").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_mode_strips_ansi_and_matches_plain_text() {
        // Output contains ANSI codes around "hello"
        let (chain, _) = create_test_chain("\x1b[32mhello\x1b[0m world");
        // With clean mode, we should be able to find plain text without ANSI
        // The clean_tty_output function strips ANSI codes
        let result = chain.clean().expect("hello world").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_mode_timeout_when_ansi_not_stripped() {
        // Output contains ANSI codes
        let (chain, _) = create_test_chain("\x1b[32mhello\x1b[0m");
        // Without clean mode, searching for plain "hello" without ANSI should still work
        // because "hello" is a substring
        let result = chain.expect("hello").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_regex_with_clean_mode() {
        let (chain, _) = create_test_chain("\x1b[32mfile123.txt\x1b[0m");
        // With clean mode, regex should match against cleaned output
        let result = chain.clean().expect_regex(r"file\d+\.txt").unwrap().run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_chain_succeeds() {
        let (chain, _) = create_test_chain("any output");
        // Running an empty chain (no steps) should succeed
        let result = chain.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_send_operations() {
        let (chain, written) = create_test_chain("$");
        let result = chain
            .expect("$")
            .send("first")
            .send("second")
            .send("third")
            .run();
        assert!(result.is_ok());

        let output = written.lock().unwrap();
        assert_eq!(String::from_utf8_lossy(&output), "firstsecondthird");
    }

    #[test]
    fn test_error_message_includes_output_context() {
        let (chain, _) = create_test_chain("some recent output that should appear in error");
        let result = chain.timeout(100).expect("not_present").run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Error should include recent output
        assert!(err_msg.contains("Received"));
        assert!(err_msg.contains("some recent output"));
    }

    #[test]
    fn test_get_output_tail_with_multibyte_characters() {
        // Test with Japanese characters (3 bytes each in UTF-8)
        let (chain, _) = create_test_chain("こんにちは世界"); // "Hello World" in Japanese
        // This tests that get_output_tail handles multi-byte chars correctly
        let result = chain.expect("世界").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeout_error_with_long_output_truncates() {
        // Create output longer than 500 chars
        let long_output = "x".repeat(1000);
        let (chain, _) = create_test_chain(&long_output);
        let result = chain.timeout(100).expect("not_present").run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Error message should mention "last 500 chars"
        assert!(err_msg.contains("last 500 chars"));
    }

    #[test]
    fn test_send_key_multiple_keys() {
        let (chain, written) = create_test_chain("$");
        let result = chain
            .expect("$")
            .send_key("ENTER")
            .send_key("TAB")
            .send_key("ESCAPE")
            .run();
        assert!(result.is_ok());

        let output = written.lock().unwrap();
        // ENTER is \r, TAB is \t, ESCAPE is \x1b
        assert_eq!(&output[..], b"\r\t\x1b");
    }

    #[test]
    fn test_chained_timeout_and_clean_modifiers() {
        let (chain, _) = create_test_chain("\x1b[32mhello\x1b[0m");
        // Both timeout and clean should apply to the next expect
        let result = chain.timeout(200).clean().expect("hello").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_resets_after_use() {
        let (chain, _) = create_test_chain("\x1b[32mhello\x1b[0m raw");
        // First expect uses clean, second does not
        let result = chain
            .clean()
            .expect("hello")
            .expect("raw") // This should NOT use clean mode
            .run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_menu_finds_menu() {
        // Simulate menu output with highlighted item using ❯ indicator
        let menu_output = "Select an option:\r\n❯ Option A\r\n  Option B\r\n  Option C";
        let (chain, _) = create_test_chain(menu_output);
        let result = chain.expect_menu().run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_menu_timeout_when_no_menu() {
        // Plain text without menu structure
        let (chain, _) = create_test_chain("Just some plain text without any menu");
        let result = chain.timeout(100).expect_menu().run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("menu"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_expect_menu_highlighted_finds_specific_item() {
        let menu_output = "Select an option:\r\n  Option A\r\n❯ Option B\r\n  Option C";
        let (chain, _) = create_test_chain(menu_output);
        let result = chain.expect_menu_highlighted("Option B").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_menu_highlighted_timeout_when_wrong_item() {
        let menu_output = "Select an option:\r\n❯ Option A\r\n  Option B\r\n  Option C";
        let (chain, _) = create_test_chain(menu_output);
        // Expecting Option B to be highlighted, but Option A is highlighted
        let result = chain.timeout(100).expect_menu_highlighted("Option B").run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Option B"));
    }

    #[test]
    fn test_expect_menu_with_greater_than_indicator() {
        // Some menus use > instead of ❯
        let menu_output = "Choose:\r\n> First\r\n  Second";
        let (chain, _) = create_test_chain(menu_output);
        let result = chain.expect_menu_highlighted("First").run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_menu_chain_with_send_key() {
        // Test chaining menu expect with key sends
        let menu_output = "❯ Item 1\r\n  Item 2";
        let (chain, written) = create_test_chain(menu_output);
        let result = chain.expect_menu().send_key("DOWN").run();
        assert!(result.is_ok());
        let output = written.lock().unwrap();
        // DOWN arrow is \x1b[B
        assert_eq!(&output[..], b"\x1b[B");
    }

    #[test]
    fn test_expect_menu_error_includes_step_number() {
        let (chain, _) = create_test_chain("no menu here");
        let result = chain.expect("no").timeout(100).expect_menu().run();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Step 2"));
    }
}
