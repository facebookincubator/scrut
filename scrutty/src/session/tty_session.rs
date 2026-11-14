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
//!
//! # Terminal protocol replies
//!
//! The PTY reader also feeds application output into a persistent [`vt100::Parser`].
//! It answers standard cursor-position queries (`ESC[6n`) through the PTY input,
//! so an inline editor can initialize without the test scripting terminal replies.
//! This parser tracks protocol state only: captured output and the existing
//! [`TerminalState`] snapshot API continue to use their original implementations.

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
/// Standard ANSI cursor-position queries are answered automatically with
/// `ESC[row;columnR`. Positions use one-based terminal cells, including Unicode
/// display widths, wrapping and scrolling. Other terminal queries are not answered.
/// Replies share the input writer with scripted keystrokes but are not appended
/// to captured output; the child's own echo behavior still applies.
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

/// Queue standard ANSI cursor-position replies in application-output order.
///
/// [`vt100::Parser`] maintains the screen and calls this handler at each query,
/// before processing subsequent output. Computing replies here, rather than after
/// a whole read, preserves distinct positions for multiple queries in one chunk.
/// The reader thread drains these bytes into the PTY input after parsing the chunk.
///
/// Only unprefixed `CSI 6 n` is supported. Private cursor queries, device status
/// and device attributes are deliberately left unanswered.
#[derive(Default)]
struct CursorReports(Vec<u8>);

impl vt100::Callbacks for CursorReports {
    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        command: char,
    ) {
        // vt100 parses the CSI parameters for us, including sequences split
        // across reads; do not scan decoded output strings for escape patterns.
        if i1.is_none() && i2.is_none() && params == [&[6][..]] && command == 'n' {
            let (row, col) = screen.cursor_position();
            // Pending wrap can put the parser one column beyond the right edge.
            // A terminal report must still name the last visible cell.
            let col = col.min(screen.size().1.get() - 1);
            // Parser coordinates are zero-based; the wire protocol is one-based.
            self.0
                .extend_from_slice(format!("\x1b[{};{}R", row + 1, col + 1).as_bytes());
        }
    }
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
        // Use the same lock as write()/write_bytes(): a complete reply batch
        // must not have scripted keystrokes inserted between its bytes.
        let reply_writer = writer.clone();
        let master = pair.master;

        // Start a thread to continuously read output from master
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            // Keep parser state across reads, including incomplete UTF-8/ANSI
            // sequences. No scrollback is needed for visible cursor reports.
            let mut terminal = vt100::Parser::new_with_callbacks(
                spawn.rows.max(1).try_into().unwrap(),
                spawn.cols.max(1).try_into().unwrap(),
                0,
                CursorReports::default(),
            );

            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        // Read actual PTY dimensions, not stale spawn options or
                        // environment variables: the child may have used stty.
                        if let Ok(size) = master.get_size() {
                            let dimensions = (
                                size.rows.max(1).try_into().unwrap(),
                                size.cols.max(1).try_into().unwrap(),
                            );
                            if terminal.screen().size() != dimensions {
                                terminal.set_size(dimensions.0, dimensions.1);
                            }
                        }
                        // Parse bytes, not independently decoded read chunks:
                        // ANSI queries and UTF-8 characters can span reads.
                        terminal.process(&buffer[..n]);
                        let replies = &mut terminal.callbacks_mut().0;
                        if !replies.is_empty() {
                            let mut writer = reply_writer.lock().unwrap();
                            // Flush promptly: the child may be blocked waiting
                            // for this report before it prints anything else.
                            if writer
                                .write_all(replies)
                                .and_then(|_| writer.flush())
                                .is_err()
                            {
                                break;
                            }
                            replies.clear();
                        }
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

#[cfg(test)]
mod cursor_report_tests {
    //! Deterministic parser tests isolate byte boundaries and cursor arithmetic
    //! from PTY scheduling. The Unix-only `pty` submodule separately exercises
    //! reply delivery and subsequent user input through real child processes.

    use super::CursorReports;

    /// A narrow screen makes wrapping and bottom-row scrolling observable.
    fn parser() -> vt100::Parser<CursorReports> {
        vt100::Parser::new_with_callbacks(
            8.try_into().unwrap(),
            12.try_into().unwrap(),
            0,
            CursorReports::default(),
        )
    }

    /// Every two-chunk split, and then byte-at-a-time delivery, must preserve both
    /// UTF-8 decoding and ANSI query state. After the full first row, `界` takes
    /// two cells and `e` plus a combining accent takes one: the cursor is (2, 4).
    ///
    /// ```text
    /// Columns: 123456789012
    /// Row 1:   123456789012
    /// Row 2:   [界][é][cursor]
    /// Cells:    1 2  3    4       -> ESC[2;4R
    ///
    /// CSI 3;7H then moves directly to row 3, column 7:
    /// Columns: 123456789012
    /// Row 3:   ......^.....       -> ESC[3;7R
    /// ```
    #[test]
    fn split_queries_and_utf8_preserve_exact_positions() {
        let bytes = "123456789012界e\u{301}\x1b[6n\x1b[3;7H\x1b[6n".as_bytes();
        for split in 0..=bytes.len() {
            let mut terminal = parser();
            terminal.process(&bytes[..split]);
            terminal.process(&bytes[split..]);
            assert_eq!(
                terminal.callbacks().0,
                b"\x1b[2;4R\x1b[3;7R",
                "split {split}"
            );
        }
        let mut terminal = parser();
        for byte in bytes {
            terminal.process(&[*byte]);
        }
        assert_eq!(terminal.callbacks().0, b"\x1b[2;4R\x1b[3;7R");
    }

    /// Replies must capture the screen at each query, not all use the final
    /// position after the trailing printable text in the same read.
    ///
    /// ```text
    /// Output in one read        Cursor (row, column)    Reply
    /// CSI 6n                    (1, 1)                  ESC[1;1R
    /// abc, CSI 6n               (1, 4)                  ESC[1;4R
    /// CSI 2;5H, CSI 6n          (2, 5)                  ESC[2;5R
    /// more                      (2, 9)                  none
    /// ```
    #[test]
    fn replies_use_position_at_each_query_not_the_end_of_the_read() {
        let mut terminal = parser();
        terminal.process(b"\x1b[6nabc\x1b[6n\x1b[2;5H\x1b[6nmore");
        assert_eq!(terminal.callbacks().0, b"\x1b[1;1R\x1b[1;4R\x1b[2;5R");
    }

    /// A query at pending wrap reports the last visible column, while a newline
    /// from the bottom row scrolls without reporting a row beyond the screen.
    ///
    /// ```text
    /// Columns: 123456789012
    /// Row 1:   123456789012      Query -> ESC[1;12R, not ESC[1;13R
    ///                     ^     Cursor remains at the right margin.
    ///
    /// At the bottom of the eight-row screen:
    ///            Before CR/LF       After CR/LF (screen scrolls)
    /// Row 7:     ............       last........
    /// Row 8:     last^.......       ^...........
    ///                               Query -> ESC[8;1R, not ESC[9;1R
    ///
    /// Dots denote blank cells; ^ marks the cursor.
    /// ```
    #[test]
    fn right_margin_and_scrolling_use_visible_terminal_coordinates() {
        let mut terminal = parser();
        terminal.process(b"123456789012\x1b[6n\r\n\x1b[8;1Hlast\r\n\x1b[6n");
        assert_eq!(terminal.callbacks().0, b"\x1b[1;12R\x1b[8;1R");
    }

    /// Device-status, private cursor and device-attribute requests stay silent;
    /// query-like text inside an OSC title is not a cursor-position request.
    /// A subsequent supported query proves parsing still works afterward.
    #[test]
    fn unsupported_queries_and_query_like_title_text_do_not_reply() {
        let mut terminal = parser();
        terminal.process(b"\x1b[5n\x1b[?6n\x1b[c\x1b]0;title[6n\x07");
        assert!(terminal.callbacks().0.is_empty());
        terminal.process(b"\x1b[6n");
        assert_eq!(terminal.callbacks().0, b"\x1b[1;1R");
    }

    #[cfg(unix)]
    mod pty {
        //! Terminal protocol replies must not require scripted user keystrokes.
        //!
        //! These tests exercise a real Unix PTY, not just the terminal parser. Bash
        //! reads the generated reports from its own input and exits unsuccessfully
        //! on a wrong coordinate or missing reply. Echo is disabled to avoid feeding
        //! replies back into output; canonical input is disabled so reports arrive
        //! without a newline. Bounded reads turn missing protocol support into a
        //! failure rather than a hang. Windows ConPTY is not exercised here.

        use super::super::Spawn;
        use super::super::TtySession;

        /// Verify automatic reports after printable output and explicit cursor
        /// movement, including two adjacent queries. Only after consuming all three
        /// replies does the child ask for user input, proving no report bytes are
        /// left in that input.
        ///
        /// ```text
        /// Child output                  Automatic PTY input        Scripted user input
        /// hello, CSI 6n                 ESC[1;6R
        /// CSI 3;7H, CSI 6n, CSI 6n      ESC[3;7R ESC[3;7R
        /// Name:                                                    Alice + newline
        /// accepted
        /// ```
        #[test]
        fn cursor_reports_follow_output_and_do_not_consume_user_input() {
            let script = r#"
set -eu
stty -echo -icanon
printf 'hello\033[6n'
IFS= read -r -t 3 -d R first
test "$first" = $'\033[1;6'
printf '\033[3;7H\033[6n\033[6n'
IFS= read -r -t 3 -d R second
IFS= read -r -t 3 -d R third
test "$second" = $'\033[3;7'
test "$third" = "$second"
printf '\r\nName: '
IFS= read -r -t 3 name
test "$name" = Alice
printf '\r\naccepted\r\n'
"#;
            let mut session = TtySession::spawn("bash", &["-c", script], ".", vec![]).unwrap();
            // Synchronize on the prompt rather than racing user input against the
            // child's protocol reads. "accepted" proves it read Alice unchanged.
            session.wait_for_output("Name:", 5000, true).unwrap();
            session.write("Alice\n").unwrap();
            session.wait_for_output("accepted", 5000, true).unwrap();
            assert_eq!(
                session.wait_for_exit(5000).unwrap(),
                0,
                "{}",
                session.get_output()
            );
            assert!(session.get_clean_output().contains("accepted"));
        }

        /// Resize from the spawn dimensions inside the child, then force wrapping
        /// with a full 12-cell row. The wide character consumes two cells on row two;
        /// `e` plus its combining accent consumes one, so the next cell is (2, 4).
        ///
        /// ```text
        /// Spawn: 24 rows x 80 columns -> child stty: 8 rows x 12 columns
        ///
        /// Columns: 123456789012
        /// Row 1:   123456789012
        /// Row 2:   [界][é][cursor]
        /// Cells:    1 2  3    4
        ///
        /// [界] spans cells 1-2; [é] occupies cell 3, not cells 3-4.
        /// Query: ESC[6n -> expected reply: ESC[2;4R
        /// ```
        #[test]
        fn cursor_reports_use_resized_pty_dimensions_and_unicode_cell_widths() {
            let script = r#"
set -eu
stty -echo -icanon cols 12 rows 8
printf '123456789012界e\314\201\033[6n'
IFS= read -r -t 3 -d R reply
test "$reply" = $'\033[2;4'
printf '\r\ncorrect\r\n'
"#;
            let mut session =
                TtySession::spawn_with(Spawn::new("bash", &["-c", script], ".").size(24, 80))
                    .unwrap();
            assert_eq!(
                session.wait_for_exit(5000).unwrap(),
                0,
                "{}",
                session.get_output()
            );
            assert!(session.get_clean_output().contains("correct"));
        }
    }
}
