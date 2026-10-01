/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Virtual screen buffer
//!
//! Maintains a 2D grid representing the terminal screen state.
//! Processes ANSI sequences to update the buffer.

use super::ansi::AnsiParser;
use super::ansi::AnsiSequence;
use super::style::Style;

/// A single cell in the screen buffer
///
/// `Copy` because rendering reads one cell at a time — the SVG pass alone asks
/// for every cell several times over — and every field is already `Copy`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Cell {
    /// The character in this cell (space if empty)
    pub ch: char,
    /// Whether this cell has been written to
    pub dirty: bool,
    /// How it was painted, for renderings that keep color
    pub style: Style,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            dirty: true,
            style: Style::default(),
        }
    }

    pub fn styled(ch: char, style: Style) -> Self {
        Self {
            ch,
            dirty: true,
            style,
        }
    }

    pub fn empty() -> Self {
        Self {
            ch: ' ',
            dirty: false,
            style: Style::default(),
        }
    }
}

/// Virtual screen buffer that simulates a terminal display
#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    /// Number of rows
    rows: usize,
    /// Number of columns
    cols: usize,
    /// The grid of cells (row-major order)
    cells: Vec<Vec<Cell>>,
    /// Current cursor row (0-indexed)
    cursor_row: usize,
    /// Current cursor column (0-indexed)
    cursor_col: usize,
    /// Saved cursor position
    saved_cursor: Option<(usize, usize)>,
    /// Appearance every subsequent write is stamped with
    style: Style,
}

impl ScreenBuffer {
    /// Create a new screen buffer with the given dimensions
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![vec![Cell::empty(); cols]; rows];
        Self {
            rows,
            cols,
            cells,
            cursor_row: 0,
            cursor_col: 0,
            saved_cursor: None,
            style: Style::default(),
        }
    }

    /// Create a buffer with default terminal size (24x80)
    pub fn default_size() -> Self {
        Self::new(24, 80)
    }

    /// Get buffer dimensions
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Screen width in character cells
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// One cell, or an empty one when the coordinates are off the screen
    pub fn cell(&self, row: usize, col: usize) -> Cell {
        self.cells
            .get(row)
            .and_then(|cells| cells.get(col))
            .cloned()
            .unwrap_or_else(Cell::empty)
    }

    /// Render the screen to SVG, keeping the colors the text loses
    pub fn to_svg(&self) -> String {
        super::svg::render(self)
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Get current cursor position (0-indexed row, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    /// Process raw terminal output and update the buffer
    pub fn process(&mut self, input: &str) {
        let sequences = AnsiParser::parse_all(input);
        for seq in sequences {
            self.apply_sequence(&seq);
        }
    }

    /// Apply a single ANSI sequence to the buffer
    fn apply_sequence(&mut self, seq: &AnsiSequence) {
        match seq {
            AnsiSequence::Text(text) => {
                for ch in text.chars() {
                    self.write_char(ch);
                }
            }
            AnsiSequence::CursorPosition { row, col } => {
                // ANSI uses 1-indexed positions
                self.cursor_row = (*row as usize).saturating_sub(1).min(self.rows - 1);
                self.cursor_col = (*col as usize).saturating_sub(1).min(self.cols - 1);
            }
            AnsiSequence::CursorUp(n) => {
                self.cursor_row = self.cursor_row.saturating_sub(*n as usize);
            }
            AnsiSequence::CursorDown(n) => {
                self.cursor_row = (self.cursor_row + *n as usize).min(self.rows - 1);
            }
            AnsiSequence::CursorForward(n) => {
                self.cursor_col = (self.cursor_col + *n as usize).min(self.cols - 1);
            }
            AnsiSequence::CursorBackward(n) => {
                self.cursor_col = self.cursor_col.saturating_sub(*n as usize);
            }
            AnsiSequence::CursorNextLine(n) => {
                self.cursor_row = (self.cursor_row + *n as usize).min(self.rows - 1);
                self.cursor_col = 0;
            }
            AnsiSequence::CursorPrevLine(n) => {
                self.cursor_row = self.cursor_row.saturating_sub(*n as usize);
                self.cursor_col = 0;
            }
            AnsiSequence::CursorColumn(col) => {
                self.cursor_col = (*col as usize).saturating_sub(1).min(self.cols - 1);
            }
            AnsiSequence::SaveCursor => {
                self.saved_cursor = Some((self.cursor_row, self.cursor_col));
            }
            AnsiSequence::RestoreCursor => {
                if let Some((row, col)) = self.saved_cursor {
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
            }
            AnsiSequence::EraseDisplay(mode) => {
                self.erase_display(*mode);
            }
            AnsiSequence::EraseLine(mode) => {
                self.erase_line(*mode);
            }
            AnsiSequence::ScrollUp(n) => {
                self.scroll_up(*n as usize);
            }
            AnsiSequence::ScrollDown(n) => {
                self.scroll_down(*n as usize);
            }
            AnsiSequence::CarriageReturn => {
                self.cursor_col = 0;
            }
            AnsiSequence::LineFeed => {
                if self.cursor_row < self.rows - 1 {
                    self.cursor_row += 1;
                } else {
                    self.scroll_up(1);
                }
            }
            AnsiSequence::Backspace => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            AnsiSequence::Tab => {
                // Move to next tab stop (every 8 columns)
                let next_tab = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next_tab.min(self.cols - 1);
            }
            // Applied, not dropped: the styling is half of what a full-screen
            // application says, and a snapshot without it cannot show either.
            AnsiSequence::Sgr(params) => self.style.apply(params),
            AnsiSequence::Bell | AnsiSequence::Unknown(_) => {}
        }
    }

    /// Write a character at the current cursor position
    fn write_char(&mut self, ch: char) {
        if self.cursor_col >= self.cols {
            // Wrap to next line
            self.cursor_col = 0;
            if self.cursor_row < self.rows - 1 {
                self.cursor_row += 1;
            } else {
                self.scroll_up(1);
            }
        }

        self.cells[self.cursor_row][self.cursor_col] = Cell::styled(ch, self.style);
        self.cursor_col += 1;
    }

    /// Erase display based on mode
    fn erase_display(&mut self, mode: u8) {
        match mode {
            0 => {
                // Cursor to end of screen
                self.erase_line(0);
                for row in (self.cursor_row + 1)..self.rows {
                    self.clear_row(row);
                }
            }
            1 => {
                // Start of screen to cursor
                for row in 0..self.cursor_row {
                    self.clear_row(row);
                }
                self.erase_line(1);
            }
            2 | 3 => {
                // Entire screen
                for row in 0..self.rows {
                    self.clear_row(row);
                }
            }
            _ => {}
        }
    }

    /// Erase line based on mode
    fn erase_line(&mut self, mode: u8) {
        match mode {
            0 => {
                // Cursor to end of line
                for col in self.cursor_col..self.cols {
                    self.cells[self.cursor_row][col] = Cell::empty();
                }
            }
            1 => {
                // Start of line to cursor
                for col in 0..=self.cursor_col {
                    self.cells[self.cursor_row][col] = Cell::empty();
                }
            }
            2 => {
                // Entire line
                self.clear_row(self.cursor_row);
            }
            _ => {}
        }
    }

    /// Clear an entire row
    fn clear_row(&mut self, row: usize) {
        for col in 0..self.cols {
            self.cells[row][col] = Cell::empty();
        }
    }

    /// Scroll the screen up by n lines
    fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            self.cells.remove(0);
            self.cells.push(vec![Cell::empty(); self.cols]);
        }
    }

    /// Scroll the screen down by n lines
    fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            self.cells.pop();
            self.cells.insert(0, vec![Cell::empty(); self.cols]);
        }
    }

    /// Get the content of a specific line (0-indexed), trimmed
    pub fn line(&self, row: usize) -> String {
        if row >= self.rows {
            return String::new();
        }

        let line: String = self.cells[row].iter().map(|c| c.ch).collect();
        line.trim_end().to_string()
    }

    /// Get all lines as strings, with trailing empty lines removed
    pub fn lines(&self) -> Vec<String> {
        let mut result: Vec<String> = self
            .cells
            .iter()
            .map(|row| {
                let line: String = row.iter().map(|c| c.ch).collect();
                line.trim_end().to_string()
            })
            .collect();

        // Remove trailing empty lines
        while result.last().map(|s| s.is_empty()).unwrap_or(false) {
            result.pop();
        }

        result
    }

    /// Get the entire screen content as a string
    pub fn content(&self) -> String {
        self.lines().join("\n")
    }

    /// Check if cursor is at the end of content (potential prompt position)
    pub fn cursor_at_content_end(&self) -> bool {
        // Check if cursor is at or after the last non-space character on its line
        let line = &self.cells[self.cursor_row];
        let last_content = line
            .iter()
            .rposition(|c| c.ch != ' ' && c.dirty)
            .map(|i| i + 1)
            .unwrap_or(0);

        self.cursor_col >= last_content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_text() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("hello");
        assert_eq!(buf.line(0), "hello");
        assert_eq!(buf.cursor_position(), (0, 5));
    }

    #[test]
    fn test_newline() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("line1\r\nline2");
        assert_eq!(buf.line(0), "line1");
        assert_eq!(buf.line(1), "line2");
    }

    #[test]
    fn test_cursor_movement() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("hello");
        buf.process("\x1b[1;1H"); // Move to 1,1 (0,0 in 0-indexed)
        buf.process("X");
        assert_eq!(buf.line(0), "Xello");
    }

    #[test]
    fn test_erase_line() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("hello");
        buf.process("\x1b[1;3H"); // Move to column 3
        buf.process("\x1b[K"); // Erase to end of line
        assert_eq!(buf.line(0), "he");
    }

    #[test]
    fn test_erase_display() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("line1\r\nline2\r\nline3");
        buf.process("\x1b[2J"); // Clear screen
        assert_eq!(buf.line(0), "");
        assert_eq!(buf.line(1), "");
        assert_eq!(buf.line(2), "");
    }

    #[test]
    fn test_carriage_return_overwrites() {
        let mut buf = ScreenBuffer::new(5, 20);
        buf.process("old text");
        buf.process("\r");
        buf.process("new");
        assert_eq!(buf.line(0), "new text");
    }

    #[test]
    fn test_wrap_at_edge() {
        let mut buf = ScreenBuffer::new(3, 5);
        buf.process("abcdefgh");
        assert_eq!(buf.line(0), "abcde");
        assert_eq!(buf.line(1), "fgh");
    }

    #[test]
    fn test_scroll_up() {
        let mut buf = ScreenBuffer::new(3, 10);
        buf.process("line1\r\nline2\r\nline3\r\nline4");
        // After scrolling, line1 should be gone
        assert_eq!(buf.line(0), "line2");
        assert_eq!(buf.line(1), "line3");
        assert_eq!(buf.line(2), "line4");
    }

    #[test]
    fn test_cursor_at_content_end() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("$ ");
        assert!(buf.cursor_at_content_end());

        buf.process("\x1b[1;1H"); // Move to start
        assert!(!buf.cursor_at_content_end());
    }

    #[test]
    fn test_save_restore_cursor() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("abc");
        buf.process("\x1b7"); // Save cursor (at 0, 3)
        buf.process("\x1b[2;5H"); // Move to row 2, col 5
        assert_eq!(buf.cursor_position(), (1, 4)); // 0-indexed
        buf.process("\x1b8"); // Restore cursor
        assert_eq!(buf.cursor_position(), (0, 3));
    }

    #[test]
    fn test_save_restore_cursor_csi() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("xyz");
        buf.process("\x1b[s"); // Save cursor (CSI version)
        buf.process("\x1b[3;1H"); // Move elsewhere
        buf.process("\x1b[u"); // Restore cursor (CSI version)
        assert_eq!(buf.cursor_position(), (0, 3));
    }

    #[test]
    fn test_scroll_down() {
        let mut buf = ScreenBuffer::new(3, 10);
        buf.process("line1\r\nline2\r\nline3");
        buf.process("\x1b[1T"); // Scroll down 1
        assert_eq!(buf.line(0), ""); // Empty line inserted at top
        assert_eq!(buf.line(1), "line1");
        assert_eq!(buf.line(2), "line2");
        // line3 scrolled off bottom
    }

    #[test]
    fn test_erase_line_mode_1() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("abcdefgh");
        buf.process("\x1b[1;5H"); // Move to column 5 (1-indexed = column 4 in 0-indexed)
        buf.process("\x1b[1K"); // Erase from start to cursor (cols 0-4 inclusive)
        // Columns 0-4 are erased (5 chars), columns 5-7 remain (fgh)
        assert_eq!(buf.line(0), "     fgh");
    }

    #[test]
    fn test_erase_line_mode_2() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("abcdefgh");
        buf.process("\x1b[1;5H"); // Move to column 5
        buf.process("\x1b[2K"); // Erase entire line
        assert_eq!(buf.line(0), "");
    }

    #[test]
    fn test_erase_display_mode_0() {
        let mut buf = ScreenBuffer::new(3, 10);
        buf.process("line1\r\nline2\r\nline3");
        buf.process("\x1b[2;3H"); // Move to row 2, col 3
        buf.process("\x1b[0J"); // Erase from cursor to end of screen
        assert_eq!(buf.line(0), "line1");
        assert_eq!(buf.line(1), "li"); // Partial line
        assert_eq!(buf.line(2), ""); // Cleared
    }

    #[test]
    fn test_erase_display_mode_1() {
        let mut buf = ScreenBuffer::new(3, 10);
        buf.process("line1\r\nline2\r\nline3");
        buf.process("\x1b[2;3H"); // Move to row 2, col 3
        buf.process("\x1b[1J"); // Erase from start of screen to cursor
        assert_eq!(buf.line(0), ""); // Cleared
        assert_eq!(buf.line(1), "   e2"); // Partial line erased
        assert_eq!(buf.line(2), "line3"); // Unchanged
    }

    #[test]
    fn test_tab_handling() {
        let mut buf = ScreenBuffer::new(5, 20);
        buf.process("a\tb");
        // Tab should move to next 8-column boundary
        assert_eq!(buf.line(0), "a       b");
    }

    #[test]
    fn test_backspace() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("abc\x08X");
        // Backspace moves cursor back, X overwrites c
        assert_eq!(buf.line(0), "abX");
    }

    #[test]
    fn test_cursor_movement_boundaries() {
        let mut buf = ScreenBuffer::new(5, 10);
        // Try to move beyond boundaries
        buf.process("\x1b[100A"); // Move up 100 rows
        assert_eq!(buf.cursor_position(), (0, 0)); // Should be clamped to 0

        buf.process("\x1b[100B"); // Move down 100 rows
        assert_eq!(buf.cursor_position(), (4, 0)); // Should be clamped to rows-1

        buf.process("\x1b[100C"); // Move forward 100 cols
        assert_eq!(buf.cursor_position(), (4, 9)); // Should be clamped to cols-1

        buf.process("\x1b[100D"); // Move backward 100 cols
        assert_eq!(buf.cursor_position(), (4, 0)); // Should be clamped to 0
    }

    #[test]
    fn test_cursor_next_line() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("abc");
        buf.process("\x1b[2E"); // Cursor to beginning of line 2 lines down
        assert_eq!(buf.cursor_position(), (2, 0));
    }

    #[test]
    fn test_cursor_prev_line() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("a\r\nb\r\nc");
        buf.process("\x1b[2F"); // Cursor to beginning of line 2 lines up
        assert_eq!(buf.cursor_position(), (0, 0));
    }

    #[test]
    fn test_cursor_column() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("abcdef");
        buf.process("\x1b[3G"); // Move to column 3 (1-indexed)
        assert_eq!(buf.cursor_position(), (0, 2)); // 0-indexed
    }

    #[test]
    fn test_dimensions() {
        let buf = ScreenBuffer::new(24, 80);
        assert_eq!(buf.dimensions(), (24, 80));
    }

    #[test]
    fn test_default_size() {
        let buf = ScreenBuffer::default_size();
        assert_eq!(buf.dimensions(), (24, 80));
    }

    #[test]
    fn test_content() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("line1\r\nline2");
        assert_eq!(buf.content(), "line1\nline2");
    }

    #[test]
    fn test_lines_removes_trailing_empty() {
        let mut buf = ScreenBuffer::new(5, 10);
        buf.process("line1\r\nline2");
        let lines = buf.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
    }

    #[test]
    fn test_line_out_of_bounds() {
        let buf = ScreenBuffer::new(3, 10);
        assert_eq!(buf.line(100), "");
    }
}
