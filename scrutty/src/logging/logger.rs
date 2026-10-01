/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Logging utilities for test output
//!
//! Provides structured logging with automatic session output flushing.
//! Uses `#[track_caller]` to capture the source location of log calls.

use std::panic::Location;

use super::SessionRegistry;

// ANSI color code for dim/grey text
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Configuration for logging behavior
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Whether to flush session output before each log message
    pub flush_sessions: bool,
    /// Whether to strip ANSI codes from session output
    pub strip_ansi: bool,
    /// Prefix to add to log messages (e.g., test name)
    pub prefix: Option<String>,
    /// Whether to show the source location (file:line)
    pub show_location: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            flush_sessions: true,
            strip_ansi: true,
            prefix: None,
            show_location: true,
        }
    }
}

/// Log a message with default configuration
///
/// This function:
/// 1. Flushes any new output from registered TTY sessions to stderr
/// 2. Prints the log message to stderr with source location
///
/// The source location (file:line) is automatically captured from where
/// this function is called, making it easy to trace log messages back
/// to the scenario script.
///
/// # Example
///
/// ```rust,ignore
/// log_message("Starting test...");
/// // Output: [scenarios/my_test.rs:42] Starting test...
/// ```
#[track_caller]
pub fn log_message(msg: &str) {
    let location = Location::caller();
    log_message_with_location(msg, &LogConfig::default(), location);
}

/// Log a message with custom configuration
///
/// # Arguments
///
/// * `msg` - The message to log
/// * `config` - Logging configuration
#[track_caller]
pub fn log_message_with_config(msg: &str, config: &LogConfig) {
    let location = Location::caller();
    log_message_with_location(msg, config, location);
}

/// Internal function that does the actual logging with a provided location
fn log_message_with_location(msg: &str, config: &LogConfig, location: &Location<'_>) {
    // Flush session output if enabled
    if config.flush_sessions {
        SessionRegistry::flush_all(config.strip_ansi);
    }

    // Format the location string (extract just the filename for cleaner output)
    let location_str = if config.show_location {
        let file_path = location.file();
        // Try to get just the relevant part of the path (from scenarios/ or src/)
        let short_path = file_path
            .rfind("scenarios/")
            .or_else(|| file_path.rfind("src/"))
            .map(|idx| &file_path[idx..])
            .unwrap_or(file_path);
        format!("{}[{}:{}]{} ", DIM, short_path, location.line(), RESET)
    } else {
        String::new()
    };

    // Format and print the message
    let formatted = match &config.prefix {
        Some(prefix) => format!("{}[{}] {}", location_str, prefix, msg),
        None => format!("{}{}", location_str, msg),
    };

    eprintln!("{}", formatted);
}
