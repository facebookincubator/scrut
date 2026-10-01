/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Piped Session - Non-interactive process execution with separate stdout/stderr
//!
//! Unlike `TtySession` which uses a PTY (merging all output into one stream),
//! `PipedSession` uses piped stdio to capture stdout and stderr separately.
//! Use this when your test runs a command and parses its output without needing
//! interactive terminal features (key input, ANSI rendering, menus).

use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;

use crate::logging::log_message;

/// Output from a completed piped session.
pub struct PipedOutput {
    /// Process exit code (-1 if the process was killed by a signal).
    pub exit_code: i32,
    /// Captured stdout as a string.
    pub stdout: String,
    /// Captured stderr as a string.
    pub stderr: String,
}

/// A non-interactive session that captures stdout and stderr separately.
///
/// Unlike `TtySession` which uses a PTY (merging all output), `PipedSession`
/// uses piped stdio, giving clean, deterministic output per stream. This avoids
/// issues where C++ warnings or stack traces on stderr pollute stdout parsing.
///
/// # Example
///
/// ```rust,ignore
/// let result = PipedSession::run("my-cli", &["--json", "status"], ".", vec![], 30_000)?;
/// assert_eq!(result.exit_code, 0);
/// let parsed: serde_json::Value = serde_json::from_str(&result.stdout)?;
/// // stderr is captured separately — noisy warnings won't break JSON parsing
/// ```
pub struct PipedSession;

impl PipedSession {
    /// Run a command and capture its output with separate stdout/stderr.
    ///
    /// Spawns the process, waits for it to complete (with timeout), and returns
    /// the captured output. Stderr is automatically logged via `log_message`
    /// if non-empty.
    ///
    /// # Arguments
    ///
    /// * `command` - Path to the executable to run
    /// * `args` - Command line arguments
    /// * `cwd` - Working directory for the process
    /// * `env` - Additional environment variables as (key, value) pairs
    /// * `timeout_ms` - Maximum time to wait for the process in milliseconds
    pub fn run(
        command: &str,
        args: &[&str],
        cwd: &str,
        env: Vec<(&str, &str)>,
        timeout_ms: u64,
    ) -> Result<PipedOutput> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in env {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().context("Failed to spawn process")?;

        // Drain stdout and stderr on background threads to avoid deadlock
        // when the child writes more than the OS pipe buffer (~64KB).
        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        let stdout_thread = std::thread::spawn(move || {
            let mut buf = String::new();
            if let Some(mut pipe) = stdout_pipe {
                let _ = pipe.read_to_string(&mut buf);
            }
            buf
        });

        let stderr_thread = std::thread::spawn(move || {
            let mut buf = String::new();
            if let Some(mut pipe) = stderr_pipe {
                let _ = pipe.read_to_string(&mut buf);
            }
            buf
        });

        // Poll for completion with timeout.
        let timeout = Duration::from_millis(timeout_ms);
        let start = Instant::now();

        let status = loop {
            match child.try_wait().context("Failed to check process status")? {
                Some(status) => break status,
                None => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(anyhow!("Process did not exit within {}ms", timeout_ms));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        };

        let stdout_buf = stdout_thread.join().unwrap_or_default();
        let stderr_buf = stderr_thread.join().unwrap_or_default();

        if !stderr_buf.is_empty() {
            log_message(&format!("stderr: {}", stderr_buf));
        }

        Ok(PipedOutput {
            exit_code: status.code().unwrap_or(-1),
            stdout: stdout_buf,
            stderr: stderr_buf,
        })
    }
}
