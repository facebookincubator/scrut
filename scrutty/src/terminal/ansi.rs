/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! ANSI escape sequence parser
//!
//! Parses ANSI/VT100 escape sequences from terminal output to understand
//! cursor movements, screen clearing, and styling operations.

/// ASCII escape character (ESC, \x1b)
const ESC: char = '\x1b';
/// ASCII backspace character (BS, \x08)
const BACKSPACE: char = '\x08';
/// ASCII bell character (BEL, \x07)
const BELL: char = '\x07';
/// ASCII carriage return (CR, \r)
const CR: char = '\r';
/// ASCII line feed (LF, \n)
const LF: char = '\n';
/// ASCII horizontal tab (HT, \t)
const TAB: char = '\t';

/// Helper to get first parameter value or a default
fn first_param(params: &[u16], default: u16) -> u16 {
    params.first().copied().unwrap_or(default)
}

/// Macro to generate cursor movement sequences with less repetition
macro_rules! cursor_seq {
    ($params:expr, $variant:ident, $default:expr) => {
        Some(AnsiSequence::$variant(first_param($params, $default)))
    };
}

/// Parsed ANSI escape sequence
#[derive(Debug, Clone, PartialEq)]
pub enum AnsiSequence {
    /// Regular text to display
    Text(String),
    /// Move cursor to absolute position (1-indexed row, col)
    CursorPosition { row: u16, col: u16 },
    /// Move cursor up N rows
    CursorUp(u16),
    /// Move cursor down N rows
    CursorDown(u16),
    /// Move cursor forward N columns
    CursorForward(u16),
    /// Move cursor backward N columns
    CursorBackward(u16),
    /// Move cursor to beginning of line N lines down
    CursorNextLine(u16),
    /// Move cursor to beginning of line N lines up
    CursorPrevLine(u16),
    /// Move cursor to column N (1-indexed)
    CursorColumn(u16),
    /// Save cursor position
    SaveCursor,
    /// Restore cursor position
    RestoreCursor,
    /// Erase display: 0=cursor to end, 1=start to cursor, 2=entire, 3=entire+scrollback
    EraseDisplay(u8),
    /// Erase line: 0=cursor to end, 1=start to cursor, 2=entire
    EraseLine(u8),
    /// Scroll up N lines
    ScrollUp(u16),
    /// Scroll down N lines
    ScrollDown(u16),
    /// Set graphics rendition (colors, bold, etc.)
    Sgr(Vec<u8>),
    /// Carriage return
    CarriageReturn,
    /// Line feed / newline
    LineFeed,
    /// Backspace
    Backspace,
    /// Tab
    Tab,
    /// Bell
    Bell,
    /// Unknown/unhandled sequence (stored for debugging)
    Unknown(String),
}

/// Parser for ANSI escape sequences
pub struct AnsiParser {
    buffer: String,
    position: usize,
}

impl AnsiParser {
    /// Create a new parser for the given input
    pub fn new(input: &str) -> Self {
        Self {
            buffer: input.to_string(),
            position: 0,
        }
    }

    /// Parse all sequences from the input
    pub fn parse_all(input: &str) -> Vec<AnsiSequence> {
        let mut parser = Self::new(input);
        let mut sequences = Vec::new();
        while let Some(seq) = parser.next_sequence() {
            sequences.push(seq);
        }
        sequences
    }

    /// Get the next sequence from the input
    pub fn next_sequence(&mut self) -> Option<AnsiSequence> {
        if self.position >= self.buffer.len() {
            return None;
        }

        let remaining = &self.buffer[self.position..];
        let first_char = remaining.chars().next()?;

        match first_char {
            ESC => self.parse_escape_sequence(),
            CR => {
                self.position += 1;
                Some(AnsiSequence::CarriageReturn)
            }
            LF => {
                self.position += 1;
                Some(AnsiSequence::LineFeed)
            }
            BACKSPACE => {
                self.position += 1;
                Some(AnsiSequence::Backspace)
            }
            TAB => {
                self.position += 1;
                Some(AnsiSequence::Tab)
            }
            BELL => {
                self.position += 1;
                Some(AnsiSequence::Bell)
            }
            _ => self.parse_text(),
        }
    }

    /// Parse an escape sequence starting with ESC
    fn parse_escape_sequence(&mut self) -> Option<AnsiSequence> {
        let remaining = &self.buffer[self.position..];

        if remaining.len() < 2 {
            // Incomplete escape, treat as text
            return self.parse_text();
        }

        let second_char = remaining.chars().nth(1)?;

        match second_char {
            '[' => self.parse_csi_sequence(),
            '7' => {
                self.position += 2;
                Some(AnsiSequence::SaveCursor)
            }
            '8' => {
                self.position += 2;
                Some(AnsiSequence::RestoreCursor)
            }
            'D' => {
                self.position += 2;
                Some(AnsiSequence::ScrollUp(1))
            }
            'M' => {
                self.position += 2;
                Some(AnsiSequence::ScrollDown(1))
            }
            ']' => self.parse_osc_sequence(),
            _ => {
                // Unknown escape sequence, consume ESC and second char
                self.position += 2;
                Some(AnsiSequence::Unknown(format!("ESC{}", second_char)))
            }
        }
    }

    /// Parse a CSI (Control Sequence Introducer) sequence: ESC [ ... final_byte
    fn parse_csi_sequence(&mut self) -> Option<AnsiSequence> {
        let remaining = &self.buffer[self.position..];

        // Find the end of the CSI sequence (a letter or @-~ range)
        let csi_start = 2; // Skip ESC [
        let mut end_pos = csi_start;
        let mut final_char: Option<char> = None;

        for ch in remaining[csi_start..].chars() {
            end_pos += ch.len_utf8();
            if ch.is_ascii_alphabetic() || ('@'..='~').contains(&ch) {
                final_char = Some(ch);
                break;
            }
        }

        if end_pos <= csi_start {
            // Incomplete CSI, treat as text
            return self.parse_text();
        }

        let final_byte = final_char?;
        let sequence = &remaining[..end_pos];
        // Extract params: everything between "[" and the final byte
        // end_pos includes the final byte, so params end at end_pos - final_byte.len_utf8()
        let params_end = end_pos - final_byte.len_utf8();
        let params_str = &remaining[csi_start..params_end];

        self.position += end_pos;

        // Parse parameters (semicolon-separated numbers)
        let params: Vec<u16> = if params_str.is_empty() {
            vec![]
        } else {
            params_str
                .split(';')
                .filter_map(|s| s.parse().ok())
                .collect()
        };

        match final_byte {
            'H' | 'f' => {
                // Cursor position
                let row = first_param(&params, 1);
                let col = params.get(1).copied().unwrap_or(1);
                Some(AnsiSequence::CursorPosition { row, col })
            }
            'A' => cursor_seq!(&params, CursorUp, 1),
            'B' => cursor_seq!(&params, CursorDown, 1),
            'C' => cursor_seq!(&params, CursorForward, 1),
            'D' => cursor_seq!(&params, CursorBackward, 1),
            'E' => cursor_seq!(&params, CursorNextLine, 1),
            'F' => cursor_seq!(&params, CursorPrevLine, 1),
            'G' => cursor_seq!(&params, CursorColumn, 1),
            'J' => Some(AnsiSequence::EraseDisplay(first_param(&params, 0) as u8)),
            'K' => Some(AnsiSequence::EraseLine(first_param(&params, 0) as u8)),
            'S' => cursor_seq!(&params, ScrollUp, 1),
            'T' => cursor_seq!(&params, ScrollDown, 1),
            'm' => {
                let sgr_params: Vec<u8> = params.iter().map(|&p| p as u8).collect();
                Some(AnsiSequence::Sgr(if sgr_params.is_empty() {
                    vec![0]
                } else {
                    sgr_params
                }))
            }
            's' => Some(AnsiSequence::SaveCursor),
            'u' => Some(AnsiSequence::RestoreCursor),
            _ => Some(AnsiSequence::Unknown(sequence.to_string())),
        }
    }

    /// Parse an OSC (Operating System Command) sequence: ESC ] ... ST
    fn parse_osc_sequence(&mut self) -> Option<AnsiSequence> {
        let remaining = &self.buffer[self.position..];

        // Find the end of the OSC sequence (BEL or ESC \)
        let mut end_pos = 2;
        let mut found_end = false;

        for (i, ch) in remaining[2..].char_indices() {
            if ch == BELL {
                end_pos = 2 + i + 1;
                found_end = true;
                break;
            }
            if ch == ESC && remaining.get(2 + i + 1..2 + i + 2) == Some("\\") {
                end_pos = 2 + i + 2;
                found_end = true;
                break;
            }
        }

        if !found_end {
            // Incomplete OSC, consume what we have
            end_pos = remaining.len();
        }

        let sequence = &remaining[..end_pos];
        self.position += end_pos;

        Some(AnsiSequence::Unknown(sequence.to_string()))
    }

    /// Parse regular text until an escape or control character
    fn parse_text(&mut self) -> Option<AnsiSequence> {
        let remaining = &self.buffer[self.position..];
        let mut text_end = 0;

        for ch in remaining.chars() {
            if ch == ESC || ch == CR || ch == LF || ch == BACKSPACE || ch == TAB || ch == BELL {
                break;
            }
            text_end += ch.len_utf8();
        }

        if text_end == 0 {
            // No text found, must be at an escape we couldn't parse
            self.position += remaining.chars().next()?.len_utf8();
            return Some(AnsiSequence::Text(remaining.chars().next()?.to_string()));
        }

        let text = remaining[..text_end].to_string();
        self.position += text_end;

        Some(AnsiSequence::Text(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let sequences = AnsiParser::parse_all("hello world");
        assert_eq!(sequences, vec![AnsiSequence::Text("hello world".into())]);
    }

    #[test]
    fn test_parse_menu_with_ansi_colors() {
        // Simulate real terminal output with ANSI color codes around menu items
        // This simulates what might come from a real interactive CLI like inquire
        let input = "\x1b[?25l\x1b[2K? Select a flavor command ›  \r\n\x1b[2K  launch\r\n\x1b[2K❯ new\r\n\x1b[2K  edit\r\n";
        let sequences = AnsiParser::parse_all(input);

        // Should parse text including the ❯ character
        let text_parts: Vec<&str> = sequences
            .iter()
            .filter_map(|s| match s {
                AnsiSequence::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();

        // Check that we have the menu indicator
        let combined = text_parts.join("");
        assert!(
            combined.contains("❯"),
            "Should contain ❯ indicator, got: {:?}",
            text_parts
        );
        assert!(
            combined.contains("new"),
            "Should contain 'new' text, got: {:?}",
            text_parts
        );
    }

    #[test]
    fn test_parse_cursor_position() {
        let sequences = AnsiParser::parse_all("\x1b[5;10H");
        assert_eq!(
            sequences,
            vec![AnsiSequence::CursorPosition { row: 5, col: 10 }]
        );
    }

    #[test]
    fn test_parse_cursor_movement() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[3A"),
            vec![AnsiSequence::CursorUp(3)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[B"),
            vec![AnsiSequence::CursorDown(1)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[2C"),
            vec![AnsiSequence::CursorForward(2)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[4D"),
            vec![AnsiSequence::CursorBackward(4)]
        );
    }

    #[test]
    fn test_parse_erase() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[2J"),
            vec![AnsiSequence::EraseDisplay(2)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[K"),
            vec![AnsiSequence::EraseLine(0)]
        );
    }

    #[test]
    fn test_parse_sgr() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[0m"),
            vec![AnsiSequence::Sgr(vec![0])]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[1;32m"),
            vec![AnsiSequence::Sgr(vec![1, 32])]
        );
    }

    #[test]
    fn test_parse_control_chars() {
        let sequences = AnsiParser::parse_all("line1\r\nline2");
        assert_eq!(
            sequences,
            vec![
                AnsiSequence::Text("line1".into()),
                AnsiSequence::CarriageReturn,
                AnsiSequence::LineFeed,
                AnsiSequence::Text("line2".into()),
            ]
        );
    }

    #[test]
    fn test_parse_mixed() {
        let sequences = AnsiParser::parse_all("\x1b[32mgreen\x1b[0m text");
        assert_eq!(
            sequences,
            vec![
                AnsiSequence::Sgr(vec![32]),
                AnsiSequence::Text("green".into()),
                AnsiSequence::Sgr(vec![0]),
                AnsiSequence::Text(" text".into()),
            ]
        );
    }

    #[test]
    fn test_parse_save_restore_cursor() {
        assert_eq!(
            AnsiParser::parse_all("\x1b7"),
            vec![AnsiSequence::SaveCursor]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b8"),
            vec![AnsiSequence::RestoreCursor]
        );
    }

    #[test]
    fn test_parse_save_restore_cursor_csi() {
        // CSI versions of save/restore
        assert_eq!(
            AnsiParser::parse_all("\x1b[s"),
            vec![AnsiSequence::SaveCursor]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[u"),
            vec![AnsiSequence::RestoreCursor]
        );
    }

    #[test]
    fn test_parse_scroll_sequences() {
        assert_eq!(
            AnsiParser::parse_all("\x1bD"),
            vec![AnsiSequence::ScrollUp(1)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1bM"),
            vec![AnsiSequence::ScrollDown(1)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[3S"),
            vec![AnsiSequence::ScrollUp(3)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[2T"),
            vec![AnsiSequence::ScrollDown(2)]
        );
    }

    #[test]
    fn test_parse_backspace() {
        let sequences = AnsiParser::parse_all("ab\x08c");
        assert_eq!(
            sequences,
            vec![
                AnsiSequence::Text("ab".into()),
                AnsiSequence::Backspace,
                AnsiSequence::Text("c".into()),
            ]
        );
    }

    #[test]
    fn test_parse_tab() {
        let sequences = AnsiParser::parse_all("a\tb");
        assert_eq!(
            sequences,
            vec![
                AnsiSequence::Text("a".into()),
                AnsiSequence::Tab,
                AnsiSequence::Text("b".into()),
            ]
        );
    }

    #[test]
    fn test_parse_bell() {
        let sequences = AnsiParser::parse_all("text\x07more");
        assert_eq!(
            sequences,
            vec![
                AnsiSequence::Text("text".into()),
                AnsiSequence::Bell,
                AnsiSequence::Text("more".into()),
            ]
        );
    }

    #[test]
    fn test_parse_cursor_next_prev_line() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[2E"),
            vec![AnsiSequence::CursorNextLine(2)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[3F"),
            vec![AnsiSequence::CursorPrevLine(3)]
        );
    }

    #[test]
    fn test_parse_cursor_column() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[5G"),
            vec![AnsiSequence::CursorColumn(5)]
        );
    }

    #[test]
    fn test_parse_cursor_position_f() {
        // 'f' is an alternative to 'H' for cursor position
        assert_eq!(
            AnsiParser::parse_all("\x1b[10;20f"),
            vec![AnsiSequence::CursorPosition { row: 10, col: 20 }]
        );
    }

    #[test]
    fn test_parse_empty_params_defaults() {
        // Empty params should default to 1
        assert_eq!(
            AnsiParser::parse_all("\x1b[H"),
            vec![AnsiSequence::CursorPosition { row: 1, col: 1 }]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[A"),
            vec![AnsiSequence::CursorUp(1)]
        );
    }

    #[test]
    fn test_parse_sgr_empty_defaults_to_reset() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[m"),
            vec![AnsiSequence::Sgr(vec![0])]
        );
    }

    #[test]
    fn test_parse_erase_modes() {
        assert_eq!(
            AnsiParser::parse_all("\x1b[0J"),
            vec![AnsiSequence::EraseDisplay(0)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[1J"),
            vec![AnsiSequence::EraseDisplay(1)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[3J"),
            vec![AnsiSequence::EraseDisplay(3)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[1K"),
            vec![AnsiSequence::EraseLine(1)]
        );
        assert_eq!(
            AnsiParser::parse_all("\x1b[2K"),
            vec![AnsiSequence::EraseLine(2)]
        );
    }

    #[test]
    fn test_parse_unknown_csi_sequence() {
        // Unknown final byte
        let sequences = AnsiParser::parse_all("\x1b[5z");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            AnsiSequence::Unknown(s) => assert!(s.contains("z")),
            _ => panic!("Expected Unknown sequence"),
        }
    }

    #[test]
    fn test_parse_osc_sequence() {
        // OSC sequence for setting title: ESC ] 0 ; title BEL
        let sequences = AnsiParser::parse_all("\x1b]0;Window Title\x07");
        assert_eq!(sequences.len(), 1);
        match &sequences[0] {
            AnsiSequence::Unknown(s) => assert!(s.contains("Window Title")),
            _ => panic!("Expected Unknown sequence for OSC"),
        }
    }

    #[test]
    fn test_parse_incomplete_escape() {
        // Single ESC at end of string - should be treated as text
        let sequences = AnsiParser::parse_all("text\x1b");
        assert!(!sequences.is_empty());
    }
}
