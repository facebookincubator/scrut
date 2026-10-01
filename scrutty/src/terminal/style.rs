/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Cell appearance, as SGR sequences describe it.
//!
//! The parser has always understood `Sgr`; the screen buffer used to drop it.
//! It is kept because a full-screen application says things with color that it
//! says nowhere else — a selected row is inverted, a highlighted region is a
//! background wash — and a snapshot without it cannot show either.

/// A color, or the terminal's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Default,
    Rgb(u8, u8, u8),
}

/// How one cell is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    /// `ESC[7m`. Applied when the cell is read rather than when it is set, so
    /// that a later `ESC[27m` is not something the buffer has to undo.
    pub reverse: bool,
}

impl Style {
    /// Foreground and background as they should actually be drawn, with
    /// `reverse` resolved and the terminal's own colors filled in.
    pub fn resolve(
        &self,
        default_fg: (u8, u8, u8),
        default_bg: (u8, u8, u8),
    ) -> ((u8, u8, u8), (u8, u8, u8)) {
        let concrete = |color: Color, fallback: (u8, u8, u8)| match color {
            Color::Default => fallback,
            Color::Rgb(r, g, b) => (r, g, b),
        };
        let fg = concrete(self.fg, default_fg);
        let bg = concrete(self.bg, default_bg);
        if self.reverse { (bg, fg) } else { (fg, bg) }
    }

    /// Folds one SGR sequence in. Unknown parameters are skipped rather than
    /// resetting anything, so an attribute this does not model cannot corrupt
    /// the ones it does.
    pub fn apply(&mut self, params: &[u8]) {
        let mut at = 0;
        while at < params.len() {
            match params[at] {
                0 => *self = Style::default(),
                1 => self.bold = true,
                22 => self.bold = false,
                7 => self.reverse = true,
                27 => self.reverse = false,
                30..=37 => {
                    self.fg = Color::Rgb(
                        ANSI_16[(params[at] - 30) as usize].0,
                        ANSI_16[(params[at] - 30) as usize].1,
                        ANSI_16[(params[at] - 30) as usize].2,
                    )
                }
                40..=47 => {
                    self.bg = Color::Rgb(
                        ANSI_16[(params[at] - 40) as usize].0,
                        ANSI_16[(params[at] - 40) as usize].1,
                        ANSI_16[(params[at] - 40) as usize].2,
                    )
                }
                90..=97 => {
                    self.fg = Color::Rgb(
                        ANSI_16[(params[at] - 90 + 8) as usize].0,
                        ANSI_16[(params[at] - 90 + 8) as usize].1,
                        ANSI_16[(params[at] - 90 + 8) as usize].2,
                    )
                }
                100..=107 => {
                    self.bg = Color::Rgb(
                        ANSI_16[(params[at] - 100 + 8) as usize].0,
                        ANSI_16[(params[at] - 100 + 8) as usize].1,
                        ANSI_16[(params[at] - 100 + 8) as usize].2,
                    )
                }
                39 => self.fg = Color::Default,
                49 => self.bg = Color::Default,
                38 | 48 => {
                    let foreground = params[at] == 38;
                    let (color, used) = extended(&params[at + 1..]);
                    if let Some(color) = color {
                        if foreground {
                            self.fg = color;
                        } else {
                            self.bg = color;
                        }
                    }
                    at += used;
                }
                _ => {}
            }
            at += 1;
        }
    }
}

/// `5;N` for the 256-color palette, `2;r;g;b` for direct color. Returns the
/// color and how many parameters past the introducer it consumed.
fn extended(rest: &[u8]) -> (Option<Color>, usize) {
    match rest.first() {
        Some(5) => match rest.get(1) {
            Some(index) => (Some(palette_256(*index)), 2),
            None => (None, rest.len()),
        },
        Some(2) => match (rest.get(1), rest.get(2), rest.get(3)) {
            (Some(r), Some(g), Some(b)) => (Some(Color::Rgb(*r, *g, *b)), 4),
            _ => (None, rest.len()),
        },
        _ => (None, 0),
    }
}

/// The xterm 256-color palette: sixteen named, a 6×6×6 cube, then a ramp.
fn palette_256(index: u8) -> Color {
    match index {
        0..=15 => {
            let (r, g, b) = ANSI_16[index as usize];
            Color::Rgb(r, g, b)
        }
        16..=231 => {
            let index = index - 16;
            let step = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
            Color::Rgb(step(index / 36), step((index % 36) / 6), step(index % 6))
        }
        _ => {
            let level = 8 + (index - 232) * 10;
            Color::Rgb(level, level, level)
        }
    }
}

/// The eight named colors and their bright counterparts, as xterm draws them.
const ANSI_16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];
