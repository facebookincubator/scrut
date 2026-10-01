/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Terminal state and menu detection
//!
//! Provides a high-level view of the terminal state including
//! cursor position, screen content, and menu detection.

use super::screen::ScreenBuffer;

/// Common prompt characters that indicate waiting for input
const PROMPT_CHARS: &[char] = &['$', '>', '?', '#', '❯', '%'];

/// Common menu selection indicators
const MENU_INDICATORS: &[&str] = &["❯", ">", "*", "→", "●", "◉", "[x]", "[X]"];

/// A detected menu in the terminal output
#[derive(Debug, Clone)]
pub struct Menu {
    /// The menu items (text content)
    items: Vec<String>,
    /// Index of the currently highlighted item (0-indexed)
    highlighted: usize,
}

impl Menu {
    /// Get all menu items
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Get the index of the highlighted item (0-indexed)
    pub fn highlighted_index(&self) -> usize {
        self.highlighted
    }

    /// Get the text of the highlighted item
    pub fn highlighted_item(&self) -> Option<&str> {
        self.items.get(self.highlighted).map(|s| s.as_str())
    }

    /// Check if the highlighted item contains the given text
    pub fn highlighted_contains(&self, text: &str) -> bool {
        self.highlighted_item()
            .map(|item| item.contains(text))
            .unwrap_or(false)
    }
}

/// A snapshot of the terminal state
#[derive(Debug, Clone)]
pub struct TerminalState {
    screen: ScreenBuffer,
}

impl TerminalState {
    /// Create a terminal state from raw output
    pub fn from_output(output: &str) -> Self {
        Self::from_output_with_size(output, 24, 80)
    }

    /// Create a terminal state with custom screen size
    pub fn from_output_with_size(output: &str, rows: usize, cols: usize) -> Self {
        let mut screen = ScreenBuffer::new(rows, cols);
        screen.process(output);
        Self { screen }
    }

    /// Get the current cursor position (0-indexed row, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        self.screen.cursor_position()
    }

    /// Get the content of a specific line (0-indexed)
    pub fn line(&self, row: usize) -> String {
        self.screen.line(row)
    }

    /// Get all non-empty lines
    pub fn lines(&self) -> Vec<String> {
        self.screen.lines()
    }

    /// One cell, with the colors it was painted in.
    ///
    /// For asserting on something an application says with color alone, which
    /// [`TerminalState::content`] cannot show at all.
    pub fn cell(&self, row: usize, col: usize) -> super::screen::Cell {
        self.screen.cell(row, col)
    }

    /// Render the screen to SVG, keeping the colors the text loses.
    ///
    /// [`TerminalState::content`] is the thing to diff; this is the thing to
    /// look at. An application says a good deal with color alone — an
    /// inverted row, a washed background over a selected region — and none of
    /// that survives being turned into text.
    pub fn to_svg(&self) -> String {
        self.screen.to_svg()
    }

    /// Write [`TerminalState::to_svg`] to a file.
    pub fn snapshot_svg(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path, self.to_svg())
    }

    /// Get the entire screen content as a string
    pub fn content(&self) -> String {
        self.screen.content()
    }

    /// Check if the terminal appears to be waiting for input
    ///
    /// Returns true if the cursor is at the end of content and
    /// the line ends with a prompt character.
    pub fn is_waiting_for_input(&self) -> bool {
        if !self.screen.cursor_at_content_end() {
            return false;
        }

        let (row, _) = self.cursor_position();
        let line = self.line(row);
        let trimmed = line.trim_end();

        // Check if line ends with a prompt character
        trimmed
            .chars()
            .last()
            .map(|ch| PROMPT_CHARS.contains(&ch))
            .unwrap_or(false)
    }

    /// Try to detect a menu in the terminal output
    ///
    /// Looks for patterns like:
    /// - Lines starting with selection indicators (❯, >, *)
    /// - Groups of similar-looking lines (options)
    ///
    /// When multiple menus are present (e.g., from screen updates),
    /// this returns the most recent (last) menu found.
    pub fn find_menu(&self) -> Option<Menu> {
        let lines = self.lines();
        if lines.is_empty() {
            return None;
        }

        // First pass: find the LAST highlighted line (most recent menu state)
        let mut highlighted_line_idx = None;
        let mut highlighted_text = String::new();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let (is_highlighted, item_text) = self.parse_menu_line(trimmed);
            if is_highlighted {
                highlighted_line_idx = Some(idx);
                highlighted_text = item_text;
                // Don't break - continue to find the last highlighted line
            }
        }

        let highlighted_line_idx = highlighted_line_idx?;

        // Second pass: look backwards from highlighted to find menu start
        let mut items_before = Vec::new();

        for idx in (0..highlighted_line_idx).rev() {
            let trimmed = lines[idx].trim();
            if trimmed.is_empty() {
                break;
            }
            // Check if this looks like an unselected menu item
            let (is_highlighted, _) = self.parse_menu_line(trimmed);
            if !is_highlighted && self.looks_like_unselected_menu_item(trimmed) {
                items_before.push(self.strip_menu_prefix(trimmed));
            } else {
                break;
            }
        }

        // Reverse items_before since we collected them backwards
        items_before.reverse();

        // Build final menu items list
        let mut menu_items = items_before;
        let highlighted_idx = menu_items.len();
        menu_items.push(highlighted_text);

        // Third pass: look forwards from highlighted to find menu end
        for line in lines.iter().skip(highlighted_line_idx + 1) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            let (is_highlighted, item_text) = self.parse_menu_line(trimmed);
            if is_highlighted {
                // Another highlighted item - add it
                menu_items.push(item_text);
            } else if self.looks_like_menu_item(trimmed, &menu_items) {
                menu_items.push(self.strip_menu_prefix(trimmed));
            } else {
                break;
            }
        }

        // Need at least 2 items to be a menu
        if menu_items.len() >= 2 {
            Some(Menu {
                items: menu_items,
                highlighted: highlighted_idx,
            })
        } else {
            None
        }
    }

    /// Check if a line looks like an unselected menu item
    fn looks_like_unselected_menu_item(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        // Check for common unselected prefixes or just indented text
        trimmed.starts_with("  ")
            || trimmed.starts_with("○ ")
            || trimmed.starts_with("◯ ")
            || trimmed.starts_with("[ ] ")
            || (!trimmed.contains(':') || !trimmed.ends_with(':'))
    }

    /// Parse a line to check if it's highlighted and extract the item text
    fn parse_menu_line(&self, line: &str) -> (bool, String) {
        for indicator in MENU_INDICATORS {
            if let Some(rest) = line.strip_prefix(indicator) {
                return (true, rest.trim().to_string());
            }
        }

        // Check for indicator with leading spaces
        let trimmed = line.trim_start();
        for indicator in MENU_INDICATORS {
            if let Some(rest) = trimmed.strip_prefix(indicator) {
                return (true, rest.trim().to_string());
            }
        }

        (false, line.to_string())
    }

    /// Check if a line looks like it could be a menu item
    fn looks_like_menu_item(&self, line: &str, existing_items: &[String]) -> bool {
        // If we have existing items, check for similar structure
        if let Some(first) = existing_items.first() {
            // Similar length (within 50%)
            let len_ratio = line.len() as f64 / first.len().max(1) as f64;
            if len_ratio < 0.5 || len_ratio > 2.0 {
                return false;
            }
        }

        // Check for common non-menu patterns
        let trimmed = line.trim();
        if trimmed.contains(':') && trimmed.ends_with(':') {
            return false; // Likely a label
        }

        // Has some content
        !trimmed.is_empty()
    }

    /// Strip menu prefix (like spaces for unselected items)
    fn strip_menu_prefix(&self, line: &str) -> String {
        let trimmed = line.trim_start();

        // Check for common unselected prefixes
        for prefix in &["  ", " ", "○ ", "◯ ", "[ ] "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                return rest.trim().to_string();
            }
        }

        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position() {
        let state = TerminalState::from_output("hello\r\nworld");
        // After "world", cursor should be at row 1, col 5
        assert_eq!(state.cursor_position(), (1, 5));
    }

    #[test]
    fn test_line_content() {
        let state = TerminalState::from_output("line1\r\nline2\r\nline3");
        assert_eq!(state.line(0), "line1");
        assert_eq!(state.line(1), "line2");
        assert_eq!(state.line(2), "line3");
    }

    #[test]
    fn test_is_waiting_for_input() {
        let state = TerminalState::from_output("$ ");
        assert!(state.is_waiting_for_input());

        let state = TerminalState::from_output("some output");
        assert!(!state.is_waiting_for_input());

        let state = TerminalState::from_output("Enter name? ");
        assert!(state.is_waiting_for_input());
    }

    #[test]
    fn test_find_menu_with_arrow() {
        let output = "Select an option:\r\n❯ Option A\r\n  Option B\r\n  Option C";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu().expect("Should find menu");

        assert_eq!(menu.items().len(), 3);
        assert_eq!(menu.highlighted_index(), 0);
        assert!(menu.highlighted_contains("Option A"));
    }

    #[test]
    fn test_find_menu_highlighted_middle() {
        let output = "  Option A\r\n❯ Option B\r\n  Option C";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu().expect("Should find menu");

        assert_eq!(menu.highlighted_index(), 1);
        assert!(menu.highlighted_contains("Option B"));
    }

    #[test]
    fn test_no_menu_in_plain_text() {
        let state = TerminalState::from_output("Just some\r\nplain text\r\nno menu here");
        assert!(state.find_menu().is_none());
    }

    #[test]
    fn test_menu_detection_with_real_flavor_output() {
        // This simulates the actual output from the flavor test
        let output =
            "? Select a flavor command ›  \r\n  launch\r\n❯ new\r\n  edit\r\n  delete\r\n  exit";
        let state = TerminalState::from_output(output);

        let lines = state.lines();
        eprintln!("Lines: {:?}", lines);

        let menu = state.find_menu();
        assert!(menu.is_some(), "Should find menu in flavor output");

        let menu = menu.unwrap();
        eprintln!("Menu items: {:?}", menu.items());
        eprintln!("Highlighted index: {}", menu.highlighted_index());
        eprintln!("Highlighted item: {:?}", menu.highlighted_item());

        assert!(
            menu.highlighted_contains("new"),
            "Highlighted item should contain 'new', got: {:?}",
            menu.highlighted_item()
        );
    }

    #[test]
    fn test_menu_detection_with_longer_output() {
        // This simulates output that accumulates over time (like the real test)
        // First, the menu shows with 'launch' highlighted
        // Then user presses DOWN and the menu redraws with 'new' highlighted
        let output = concat!(
            "INFO Some tip\r\n",
            "? Select a flavor command ›  \r\n",
            "❯ launch\r\n",
            "  new\r\n",
            "  edit\r\n",
            "  delete\r\n",
            "  exit\r\n",
            "Some warning text\r\n",
            "? Select a flavor command ›  \r\n",
            "  launch\r\n",
            "❯ new\r\n",
            "  edit\r\n",
            "  delete\r\n",
            "  exit\r\n",
            "WARNING: Ongoing issue\r\n"
        );
        let state = TerminalState::from_output(output);

        let lines = state.lines();
        eprintln!("Lines (longer output): {:?}", lines);

        let menu = state.find_menu();
        assert!(menu.is_some(), "Should find menu in longer output");

        let menu = menu.unwrap();
        eprintln!("Menu items: {:?}", menu.items());
        eprintln!("Highlighted index: {}", menu.highlighted_index());
        eprintln!("Highlighted item: {:?}", menu.highlighted_item());

        // The most recent menu should have 'new' highlighted
        assert!(
            menu.highlighted_contains("new"),
            "Highlighted item should contain 'new', got: {:?}",
            menu.highlighted_item()
        );
    }

    #[test]
    fn test_content() {
        let state = TerminalState::from_output("line1\r\nline2");
        assert_eq!(state.content(), "line1\nline2");
    }

    #[test]
    fn test_lines() {
        let state = TerminalState::from_output("line1\r\nline2\r\nline3");
        let lines = state.lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_from_output_with_size() {
        let state = TerminalState::from_output_with_size("hello", 10, 40);
        assert_eq!(state.line(0), "hello");
    }

    #[test]
    fn test_menu_items() {
        let output = "❯ First\r\n  Second\r\n  Third";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu().unwrap();
        assert_eq!(menu.items().len(), 3);
        assert_eq!(menu.items()[0], "First");
        assert_eq!(menu.items()[1], "Second");
        assert_eq!(menu.items()[2], "Third");
    }

    #[test]
    fn test_menu_highlighted_index() {
        let output = "  First\r\n  Second\r\n❯ Third";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu().unwrap();
        assert_eq!(menu.highlighted_index(), 2);
    }

    #[test]
    fn test_menu_highlighted_item() {
        let output = "  Item A\r\n❯ Item B\r\n  Item C";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu().unwrap();
        assert_eq!(menu.highlighted_item(), Some("Item B"));
    }

    #[test]
    fn test_menu_highlighted_contains() {
        let output = "❯ Delete all files\r\n  Keep files";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu().unwrap();
        assert!(menu.highlighted_contains("Delete"));
        assert!(menu.highlighted_contains("all"));
        assert!(!menu.highlighted_contains("Keep"));
    }

    #[test]
    fn test_is_waiting_for_input_various_prompts() {
        // Test various prompt characters
        assert!(TerminalState::from_output("> ").is_waiting_for_input());
        assert!(TerminalState::from_output("# ").is_waiting_for_input());
        assert!(TerminalState::from_output("% ").is_waiting_for_input());
        assert!(TerminalState::from_output("❯ ").is_waiting_for_input());
    }

    #[test]
    fn test_is_waiting_for_input_not_at_end() {
        // Cursor not at end of content
        let state = TerminalState::from_output("$ \x1b[1;1H"); // Cursor moved to start
        assert!(!state.is_waiting_for_input());
    }

    #[test]
    fn test_find_menu_with_asterisk_indicator() {
        let output = "* Selected\r\n  Other";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu();
        assert!(menu.is_some());
        assert!(menu.unwrap().highlighted_contains("Selected"));
    }

    #[test]
    fn test_find_menu_with_checkbox_indicator() {
        let output = "[x] Checked item\r\n[ ] Unchecked item";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu();
        assert!(menu.is_some());
    }

    #[test]
    fn test_find_menu_single_item_returns_none() {
        // A single item is not a menu (need at least 2)
        let output = "❯ Only one item";
        let state = TerminalState::from_output(output);
        assert!(state.find_menu().is_none());
    }

    #[test]
    fn test_find_menu_empty_output() {
        let state = TerminalState::from_output("");
        assert!(state.find_menu().is_none());
    }

    #[test]
    fn test_find_menu_with_circle_indicators() {
        let output = "❯ Active\r\n○ Inactive";
        let state = TerminalState::from_output(output);
        let menu = state.find_menu();
        assert!(menu.is_some());
        let menu = menu.unwrap();
        assert_eq!(menu.items().len(), 2);
    }

    #[test]
    fn test_line_empty() {
        let state = TerminalState::from_output("");
        assert_eq!(state.line(0), "");
        assert_eq!(state.line(10), "");
    }

    #[test]
    fn test_cursor_after_ansi_sequences() {
        // Test cursor position after processing ANSI sequences
        let state = TerminalState::from_output("\x1b[32mcolored\x1b[0m text");
        // Should have "colored text" on first line
        assert!(state.line(0).contains("colored"));
    }
}
