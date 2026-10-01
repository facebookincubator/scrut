/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Session output registry for continuous logging
//!
//! When TTY sessions are registered, their output is periodically flushed
//! to stderr when `log_message()` is called.

use std::sync::Arc;
use std::sync::Mutex;

use crate::terminal::ScreenBuffer;

// ANSI color codes for terminal output
const GREY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Global registry tracking (output_buffer, last_flushed_position) for each session
static REGISTRY: Mutex<Vec<(Arc<Mutex<String>>, Arc<Mutex<usize>>)>> = Mutex::new(Vec::new());

/// Registry for tracking TTY sessions for continuous output logging
///
/// When a session is registered, its new output will be flushed to stderr
/// each time `log_message()` is called. This provides visibility into
/// what the CLI is outputting during test execution.
pub struct SessionRegistry;

impl SessionRegistry {
    /// Register a session's output buffer for logging
    ///
    /// Call this method to have the session's output periodically flushed
    /// when `log_message()` is called.
    pub fn register(output: Arc<Mutex<String>>) {
        let last_position = Arc::new(Mutex::new(0));
        let mut registry = REGISTRY.lock().unwrap();
        registry.push((output, last_position));
    }

    /// Flush any new output from all registered sessions to stderr
    ///
    /// This is called automatically by `log_message()`, but can also be
    /// called directly if needed.
    ///
    /// # Arguments
    ///
    /// * `strip_ansi` - If true, process output through terminal emulator to
    ///   handle carriage returns and ANSI sequences properly
    pub fn flush_all(strip_ansi: bool) {
        let registry = REGISTRY.lock().unwrap();

        for (output, last_position) in registry.iter() {
            let current_output = output.lock().unwrap();
            let mut last_pos = last_position.lock().unwrap();

            if current_output.len() > *last_pos {
                let new_output = &current_output[*last_pos..];
                *last_pos = current_output.len();

                if !new_output.trim().is_empty() {
                    let display_output = if strip_ansi {
                        // Use ScreenBuffer to properly handle carriage returns,
                        // ANSI escape sequences, and spinner animations
                        let mut screen = ScreenBuffer::new(100, 120);
                        screen.process(new_output);
                        screen.content()
                    } else {
                        new_output.to_string()
                    };

                    if !display_output.trim().is_empty() {
                        eprintln!("{}{}", GREY, "─".repeat(80));
                        eprintln!("SESSION OUTPUT:");
                        eprintln!("{}", "─".repeat(80));
                        eprintln!("{}", display_output);
                        eprintln!("{}{}", "─".repeat(80), RESET);
                    }
                }
            }
        }
    }

    /// Clear all registered sessions
    ///
    /// This is useful for cleanup between tests.
    pub fn clear() {
        let mut registry = REGISTRY.lock().unwrap();
        registry.clear();
    }

    /// Get the number of registered sessions
    pub fn count() -> usize {
        REGISTRY.lock().unwrap().len()
    }
}
