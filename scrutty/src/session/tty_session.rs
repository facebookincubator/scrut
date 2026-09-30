/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! TTY Session - Pseudo-terminal session for interactive CLI testing
//!
//! This module provides the core `TtySession` type for spawning and interacting
//! with CLI processes through a pseudo-terminal. Uses `portable-pty` for
//! cross-platform support (Unix PTY and Windows ConPTY).

use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use portable_pty::Child;
use portable_pty::CommandBuilder;
use portable_pty::PtySize;
use portable_pty::native_pty_system;
use regex::Regex;

use super::expect::ExpectChain;
use super::keys::key_name_to_sequence;
use super::output::clean_tty_output;
use super::output::strip_ansi_codes;
use crate::logging::SessionRegistry;
use crate::terminal::TerminalState;

fn floor_char_boundary(s: &str, mut index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    while index > 0 && !s.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Terminal size [`TtySession::spawn`] opens a pty at.
pub const DEFAULT_ROWS: u16 = 24;
/// See [`DEFAULT_ROWS`].
pub const DEFAULT_COLS: u16 = 80;

/// What to spawn, and the terminal to spawn it on.
///
/// A struct rather than six positional arguments, so that a call site cannot
/// silently transpose `rows` and `cols` — or `command` and `cwd` — and so that
/// further pty options can be added without touching every caller.
pub struct Spawn<'a> {
    /// Path to the executable to run.
    pub command: &'a str,
    /// Command line arguments.
    pub args: &'a [&'a str],
    /// Working directory for the process.
    pub cwd: &'a str,
    /// Additional environment variables, as `(key, value)` pairs.
    pub env: Vec<(&'a str, &'a str)>,
    /// Terminal height in character cells.
    pub rows: u16,
    /// Terminal width in character cells.
    pub cols: u16,
}

impl<'a> Spawn<'a> {
    /// A spawn on the default [`DEFAULT_ROWS`] by [`DEFAULT_COLS`] terminal,
    /// with no extra environment.
    pub fn new(command: &'a str, args: &'a [&'a str], cwd: &'a str) -> Self {
        Self {
            command,
            args,
            cwd,
            env: Vec::new(),
            rows: DEFAULT_ROWS,
            cols: DEFAULT_COLS,
        }
    }

    /// Environment variables to add to the inherited ones.
    pub fn env(mut self, env: Vec<(&'a str, &'a str)>) -> Self {
        self.env = env;
        self
    }

    /// The terminal size the pty is opened at.
    pub fn size(mut self, rows: u16, cols: u16) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }
}

/// A pseudo-terminal session for interacting with CLI processes
///
/// `TtySession` spawns a child process attached to a PTY, allowing you to:
/// - Send input (including special keys like arrows, ctrl sequences)
/// - Read output as it arrives
/// - Wait for specific output patterns
/// - Track exit codes
///
/// This implementation uses `portable-pty` for cross-platform support,
/// working on both Unix (via PTY) and Windows (via ConPTY).
///
/// # Example
///
/// ```rust,ignore
/// let mut session = TtySession::spawn("bash", &["-c", "echo hello"], ".", vec![])?;
/// session.wait_for_output("hello", 5000, false)?;
/// let exit_code = session.wait_for_exit(5000)?;
/// assert_eq!(exit_code, 0);
/// ```
pub struct TtySession {
    child: Option<Box<dyn Child + Send + Sync>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output: Arc<Mutex<String>>,
    _reader_thread: Option<std::thread::JoinHandle<()>>,
}

impl TtySession {
    /// Spawn a new TTY session
    ///
    /// # Arguments
    ///
    /// * `command` - Path to the executable to run
    /// * `args` - Command line arguments
    /// * `cwd` - Working directory for the process
    /// * `env` - Additional environment variables as (key, value) pairs
    ///
    /// # Returns
    ///
    /// A new `TtySession` connected to the spawned process
    pub fn spawn(command: &str, args: &[&str], cwd: &str, env: Vec<(&str, &str)>) -> Result<Self> {
        Self::spawn_with(Spawn::new(command, args, cwd).env(env))
    }

    /// Spawn a new TTY session on a terminal of a given size
    ///
    /// Same as [`TtySession::spawn`], which uses [`DEFAULT_ROWS`] by
    /// [`DEFAULT_COLS`]. A full-screen application reads its layout from the
    /// pty rather than from `LINES` and `COLUMNS`, so a test for one has to say
    /// in the [`Spawn`] how much room it is being given.
    pub fn spawn_with(spawn: Spawn<'_>) -> Result<Self> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: spawn.rows,
                cols: spawn.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;

        // Build the command
        let mut cmd = CommandBuilder::new(spawn.command);
        cmd.args(spawn.args);
        cmd.cwd(spawn.cwd);
        cmd.env("TERM", "xterm-256color");

        for (key, value) in spawn.env {
            cmd.env(key, value);
        }

        // Spawn the child process
        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn child process")?;

        // Get reader and writer from master
        let mut reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        let output = Arc::new(Mutex::new(String::new()));
        let output_clone = output.clone();
        let writer = Arc::new(Mutex::new(writer));

        // Start a thread to continuously read output from master
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if let Ok(s) = String::from_utf8(buffer[..n].to_vec()) {
                            let mut out = output_clone.lock().unwrap();
                            out.push_str(&s);
                        }
                    }
                    _ => break,
                }
            }
        });

        // Give the process a moment to initialize
        std::thread::sleep(Duration::from_millis(200));

        Ok(TtySession {
            child: Some(child),
            writer,
            output,
            _reader_thread: Some(reader_thread),
        })
    }

    /// Get the current accumulated output (raw, with ANSI codes)
    pub fn get_output(&self) -> String {
        self.output.lock().unwrap().clone()
    }

    /// Get the current output with ANSI codes stripped
    pub fn get_clean_output(&self) -> String {
        strip_ansi_codes(&self.get_output())
    }

    /// Get the current output cleaned for menu/interactive display
    pub fn get_display_output(&self) -> String {
        clean_tty_output(&self.get_output())
    }

    /// Write raw data to the session
    pub fn write(&self, data: &str) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer
            .write_all(data.as_bytes())
            .context("Failed to write to PTY")?;
        writer.flush()?;
        Ok(())
    }

    /// Write raw bytes to the session, e.g. single non-UTF-8 bytes from
    /// `#xx` hex key tokens that cannot be represented as `&str`.
    pub fn write_bytes(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(data).context("Failed to write to PTY")?;
        writer.flush()?;
        Ok(())
    }

    /// Send a single key to the session
    ///
    /// Accepts either raw escape sequences or human-readable names like "ENTER", "CTRL+C"
    pub fn send_key(&self, key: &str) -> Result<()> {
        let sequence = key_name_to_sequence(key);
        self.write(sequence)?;
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    /// Send multiple keys to the session
    ///
    /// Each key can be a raw escape sequence or a human-readable name.
    /// There's a small delay between keys.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// session.send_keys(&["DOWN", "DOWN", "ENTER"])?;
    /// session.send_keys(&[Keys::CTRL_C])?;
    /// session.send_keys(&["hello", "ENTER"])?;  // Types "hello" then presses enter
    /// ```
    pub fn send_keys(&self, keys: &[&str]) -> Result<()> {
        for key in keys {
            self.send_key(key)?;
        }
        Ok(())
    }

    /// Wait for specific text to appear in the output
    ///
    /// # Arguments
    ///
    /// * `expected` - The text to wait for
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    /// * `use_clean_output` - If true, strip ANSI codes before checking
    ///
    /// # Returns
    ///
    /// The output at the time the expected text was found
    pub fn wait_for_output(
        &self,
        expected: &str,
        timeout_ms: u64,
        use_clean_output: bool,
    ) -> Result<String> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() >= timeout {
                let output = self.get_output();
                let display_output = if use_clean_output {
                    clean_tty_output(&output)
                } else {
                    output.clone()
                };

                let tail_start =
                    floor_char_boundary(&display_output, display_output.len().saturating_sub(500));
                let tail = &display_output[tail_start..];
                return Err(anyhow!(
                    "Timeout waiting for output.\nExpected: {}\nReceived (last 500 chars): {}",
                    expected,
                    tail
                ));
            }

            let current_output = self.get_output();
            let check_output = if use_clean_output {
                clean_tty_output(&current_output)
            } else {
                current_output.clone()
            };

            if check_output.contains(expected) {
                return Ok(check_output);
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Wait for output matching a regex pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - The regex pattern to match
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    /// * `use_clean_output` - If true, strip ANSI codes before checking
    ///
    /// # Returns
    ///
    /// The output at the time the pattern was matched
    pub fn wait_for_regex(
        &self,
        pattern: &Regex,
        timeout_ms: u64,
        use_clean_output: bool,
    ) -> Result<String> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() >= timeout {
                let output = self.get_output();
                let display_output = if use_clean_output {
                    clean_tty_output(&output)
                } else {
                    output.clone()
                };

                let tail_start =
                    floor_char_boundary(&display_output, display_output.len().saturating_sub(500));
                let tail = &display_output[tail_start..];
                return Err(anyhow!(
                    "Timeout waiting for pattern.\nPattern: {}\nReceived (last 500 chars): {}",
                    pattern,
                    tail
                ));
            }

            let current_output = self.get_output();
            let check_output = if use_clean_output {
                clean_tty_output(&current_output)
            } else {
                current_output.clone()
            };

            if pattern.is_match(&check_output) {
                return Ok(check_output);
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Wait for a prompt character (? or ❯)
    ///
    /// This is a convenience method for interactive CLI menus.
    pub fn wait_for_prompt(&self, timeout_ms: u64) -> Result<String> {
        let pattern = Regex::new(r"[?❯]")?;
        self.wait_for_regex(&pattern, timeout_ms, false)
    }

    /// Start a fluent expect chain
    ///
    /// Returns an `ExpectChain` builder that allows chaining expect/send
    /// operations. Call `.run()` to execute the chain.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// session.expect("$")
    ///     .send("echo hello")
    ///     .send_key(Keys::ENTER)
    ///     .expect("hello")
    ///     .run()?;
    /// ```
    pub fn expect(&self, text: &str) -> ExpectChain<'_> {
        ExpectChain::new(self.output.clone(), self.writer.clone()).expect(text)
    }

    /// Start a fluent expect chain with a regex pattern
    ///
    /// Like `expect()` but accepts a regex pattern instead of literal text.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// session.expect_regex(r"\$ ")?
    ///     .send("ls")
    ///     .send_key(Keys::ENTER)
    ///     .expect_regex(r"file\.txt")?
    ///     .run()?;
    /// ```
    pub fn expect_regex_chain(&self, pattern: &str) -> Result<ExpectChain<'_>> {
        ExpectChain::new(self.output.clone(), self.writer.clone()).expect_regex(pattern)
    }

    /// Start a fluent chain with a send action
    ///
    /// Use this when you don't need to wait for initial output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// session.chain()
    ///     .send("command")
    ///     .send_key(Keys::ENTER)
    ///     .expect("output")
    ///     .run()?;
    /// ```
    pub fn chain(&self) -> ExpectChain<'_> {
        ExpectChain::new(self.output.clone(), self.writer.clone())
    }

    /// Wait for the process to exit
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    ///
    /// # Returns
    ///
    /// The process exit code
    pub fn wait_for_exit(&mut self, timeout_ms: u64) -> Result<i32> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        if let Some(ref mut child) = self.child {
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        return Ok(status.exit_code() as i32);
                    }
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            return Err(anyhow!("Process did not exit within {}ms", timeout_ms));
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => return Err(anyhow!("Failed to wait for process: {}", e)),
                }
            }
        }

        Err(anyhow!("No child process"))
    }

    /// Check if the process has exited
    pub fn has_exited(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            matches!(child.try_wait(), Ok(Some(_)))
        } else {
            true
        }
    }

    /// Close the session, killing the process if still running
    pub fn close(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// Register this session for continuous output logging
    ///
    /// When registered, the logging system will periodically flush new output
    /// from this session when `log_message()` is called.
    pub fn register_for_logging(&self) {
        SessionRegistry::register(self.output.clone());
    }

    /// Get a snapshot of the current terminal state
    ///
    /// Returns a `TerminalState` that provides:
    /// - Cursor position
    /// - Line content access
    /// - Prompt detection
    /// - Menu detection
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state = session.terminal_state();
    /// let (row, col) = state.cursor_position();
    /// let current_line = state.line(row);
    ///
    /// if let Some(menu) = state.find_menu() {
    ///     println!("Highlighted: {:?}", menu.highlighted_item());
    /// }
    /// ```
    pub fn terminal_state(&self) -> TerminalState {
        let output = self.get_output();
        TerminalState::from_output(&output)
    }

    /// Get a reference to the output buffer for external logging integration
    pub fn output_buffer(&self) -> Arc<Mutex<String>> {
        self.output.clone()
    }
}

impl Drop for TtySession {
    fn drop(&mut self) {
        self.close();
    }
}
