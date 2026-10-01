/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Output cleaning utilities for TTY sessions
//!
//! Terminal output often contains ANSI escape codes for colors, cursor movement,
//! and other formatting. This module provides utilities to clean this output
//! for easier assertion and logging.

use regex::Regex;

/// Strip ANSI escape codes from terminal output
///
/// Removes:
/// - CSI (Control Sequence Introducer) sequences like cursor movement, colors
/// - SGR (Select Graphic Rendition) sequences for text styling
/// - OSC (Operating System Command) sequences for window titles, etc.
/// - Other control characters (except newlines and tabs)
///
/// # Example
///
/// ```rust,ignore
/// let raw = "\x1b[32mHello\x1b[0m World";
/// let clean = strip_ansi_codes(raw);
/// assert_eq!(clean, "Hello World");
/// ```
pub fn strip_ansi_codes(output: &str) -> String {
    let mut cleaned = output.to_string();

    // Strip CSI sequences (cursor movement, colors, etc.)
    // Matches: ESC [ (optional ?>=) digits/semicolons letter
    if let Ok(re) = Regex::new(r"\x1b\[[?>=]?[0-9;]*[a-zA-Z]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip SGR sequences (text styling)
    // Matches: ESC [ digits/semicolons m
    if let Ok(re) = Regex::new(r"\x1b\[[0-9;]*m") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip OSC sequences (window title, etc.)
    // Matches: ESC ] ... (ESC \ or BEL)
    if let Ok(re) = Regex::new(r"\x1b\][^\x1b\x07]*(\x1b\\|\x07)") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Strip other control characters except newlines (\n), carriage returns (\r), and tabs (\t)
    // Keeps: 0x09 (tab), 0x0A (newline), 0x0D (carriage return)
    if let Ok(re) = Regex::new(r"[\x00-\x08\x0B-\x0C\x0E-\x1F\x7F]") {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Clean up multiple consecutive newlines (more than 2)
    if let Ok(re) = Regex::new(r"\n{3,}") {
        cleaned = re.replace_all(&cleaned, "\n\n").to_string();
    }

    cleaned.trim().to_string()
}

/// Clean TTY output to show only the last "screen"
///
/// Interactive CLI applications often redraw the screen, which results in
/// accumulated output containing multiple versions of the same content.
/// This function extracts only the final screen state by handling page
/// clear sequences.
///
/// For menu detection and interaction, use `TerminalState::find_menu()` instead.
///
/// # Example
///
/// ```rust,ignore
/// let raw = "... old content ...\x1b[2J... final screen ...";
/// let clean = clean_tty_output(raw);
/// // clean contains only "... final screen ..."
/// ```
pub fn clean_tty_output(output: &str) -> String {
    let mut cleaned = output.to_string();

    // Handle page clear patterns before stripping escape codes
    // These indicate a screen redraw, so we only want content after the last clear
    let clear_patterns = [
        "\x0C",          // Form feed
        "\x1b[2J",       // Clear entire screen
        "\x1b[H\x1b[2J", // Move to home and clear
    ];

    for pattern in &clear_patterns {
        let parts: Vec<&str> = cleaned.split(pattern).collect();
        if parts.len() > 1 {
            cleaned = parts[parts.len() - 1].to_string();
        }
    }

    // Strip ANSI codes
    cleaned = strip_ansi_codes(&cleaned);

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes_colors() {
        let input = "\x1b[32mGreen\x1b[0m Normal \x1b[1;31mBold Red\x1b[0m";
        assert_eq!(strip_ansi_codes(input), "Green Normal Bold Red");
    }

    #[test]
    fn test_strip_ansi_codes_cursor() {
        let input = "\x1b[2J\x1b[HHello\x1b[5;10HWorld";
        assert_eq!(strip_ansi_codes(input), "HelloWorld");
    }

    #[test]
    fn test_clean_tty_output_page_clear() {
        let input = "old content\x1b[2Jnew content";
        assert_eq!(clean_tty_output(input), "new content");
    }
}
