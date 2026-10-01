/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Rendering a screen to SVG, so a snapshot keeps its colors.
//!
//! Plain text is the better thing to diff, and the worse thing to look at: a
//! selected row and a highlighted region are both color and nothing else, so
//! in text they are invisible. This draws what was actually on screen.
//!
//! Cells are a fixed size. A terminal can be asked to draw double-width or
//! double-height lines, and nothing that a test drives is likely to.

use std::fmt;
use std::fmt::Write as _;

use super::screen::ScreenBuffer;
use super::style::Style;

/// Cell size in pixels, and where the baseline sits inside one. Chosen so the
/// glyphs of a 14px monospace font sit in the box without touching its edges.
const CELL_W: usize = 8;
const CELL_H: usize = 17;
const BASELINE: usize = 13;
const FONT_SIZE: usize = 14;

/// Stand-ins for "whatever the terminal uses", picked dark because that is what
/// a full-screen application is drawn against and what its palettes assume.
const DEFAULT_FG: (u8, u8, u8) = (216, 216, 216);
const DEFAULT_BG: (u8, u8, u8) = (28, 28, 28);

/// A font stack rather than one name: this is opened by whoever is reviewing,
/// and the box-drawing glyphs have to come from somewhere.
const FONT: &str = "'DejaVu Sans Mono','Menlo','Consolas',monospace";

pub(crate) fn render(screen: &ScreenBuffer) -> String {
    let (rows, cols) = (screen.rows(), screen.cols());
    let (width, height) = (cols * CELL_W, rows * CELL_H);

    let mut svg = String::with_capacity(rows * cols * 8);
    // Written into rather than formatted and appended: the loops below run once
    // per cell, and a `format!` each would allocate a string only to copy it in
    // and drop it. Writing to a `String` cannot fail.
    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" \
         viewBox=\"0 0 {width} {height}\" font-family=\"{FONT}\" font-size=\"{FONT_SIZE}px\">\n"
    );
    let _ = write!(
        svg,
        "<rect width=\"{width}\" height=\"{height}\" fill=\"{}\"/>\n",
        Hex(DEFAULT_BG)
    );

    // Backgrounds first, as one rect per run rather than per cell: a screen is
    // mostly its default color, and a rect each would be tens of thousands.
    for row in 0..rows {
        for (from, to, style) in runs(screen, row, |style| style.resolve(DEFAULT_FG, DEFAULT_BG).1)
        {
            let bg = style.resolve(DEFAULT_FG, DEFAULT_BG).1;
            if bg == DEFAULT_BG {
                continue;
            }
            let _ = write!(
                svg,
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{CELL_H}\" fill=\"{}\"/>\n",
                from * CELL_W,
                row * CELL_H,
                (to - from) * CELL_W,
                Hex(bg)
            );
        }
    }

    for row in 0..rows {
        for col in 0..cols {
            let cell = screen.cell(row, col);
            let Some((x, y, w, h)) = block(cell.ch) else {
                continue;
            };
            let (fg, _) = cell.style.resolve(DEFAULT_FG, DEFAULT_BG);
            let _ = write!(
                svg,
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                (col as f64 + x) * CELL_W as f64,
                (row as f64 + y) * CELL_H as f64,
                w * CELL_W as f64,
                h * CELL_H as f64,
                Hex(fg)
            );
        }
    }

    for row in 0..rows {
        for (from, to, style) in runs(screen, row, |style| {
            (style.resolve(DEFAULT_FG, DEFAULT_BG).0, style.bold)
        }) {
            let text: String = (from..to)
                .map(|col| screen.cell(row, col).ch)
                .map(|ch| if block(ch).is_some() { ' ' } else { ch })
                .collect();
            if text.trim().is_empty() {
                continue;
            }
            let (fg, _) = style.resolve(DEFAULT_FG, DEFAULT_BG);
            let weight = if style.bold {
                " font-weight=\"bold\""
            } else {
                ""
            };
            // `textLength` pins the run to its columns, so alignment survives
            // whichever of the fallback fonts the reader happens to have.
            let _ = write!(
                svg,
                "<text x=\"{}\" y=\"{}\" fill=\"{}\"{weight} textLength=\"{}\" \
                 lengthAdjust=\"spacingAndGlyphs\" xml:space=\"preserve\">{}</text>\n",
                from * CELL_W,
                row * CELL_H + BASELINE,
                Hex(fg),
                (to - from) * CELL_W,
                Escaped(&text)
            );
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Splits one row into maximal runs of cells that `key` agrees on.
fn runs<K: PartialEq>(
    screen: &ScreenBuffer,
    row: usize,
    key: impl Fn(&Style) -> K,
) -> Vec<(usize, usize, Style)> {
    let mut out = Vec::new();
    let mut from = 0;
    while from < screen.cols() {
        let style = screen.cell(row, from).style;
        let mut to = from + 1;
        while to < screen.cols() && key(&screen.cell(row, to).style) == key(&style) {
            to += 1;
        }
        out.push((from, to, style));
        from = to;
    }
    out
}

/// Where a block element's ink belongs, as `(x, y, width, height)` fractions of
/// one cell, or `None` for a character the font should draw.
///
/// Drawn rather than typeset because a glyph goes where the font's outline puts
/// it and fonts disagree: U+2588 measures 1.03em, 1.13em and 1.19em in the
/// three this was checked against, so at any one cell height a stacked column
/// is chipped for some readers and smeared over its neighbor for others.
///
/// The whole geometric run is covered, not the handful a chart happens to use.
/// Quadrants and shades are left out: they need more than one rect, and a shade
/// drawn as flat opacity would be a different picture from the one on screen.
fn block(ch: char) -> Option<(f64, f64, f64, f64)> {
    let eighth = |n: u32| f64::from(n) / 8.0;
    Some(match ch as u32 {
        0x2580 => (0.0, 0.0, 1.0, 0.5),
        0x2590 => (0.5, 0.0, 0.5, 1.0),
        0x2594 => (0.0, 0.0, 1.0, eighth(1)),
        0x2595 => (1.0 - eighth(1), 0.0, eighth(1), 1.0),
        // U+2581..=U+2588 fill from the bottom, an eighth at a time, up to the
        // full block; U+2589..=U+258F fill from the left, an eighth at a time.
        cp @ 0x2581..=0x2588 => (0.0, 1.0 - eighth(cp - 0x2580), 1.0, eighth(cp - 0x2580)),
        cp @ 0x2589..=0x258F => (0.0, 0.0, eighth(0x2590 - cp), 1.0),
        _ => return None,
    })
}

/// `#rrggbb`, written straight into the output. A `String` per color would be
/// one allocation per background run, per block cell and per text run.
struct Hex((u8, u8, u8));

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

/// XML-escaped, in one pass. Chaining `replace` walks the text once and
/// allocates a fresh string per character being escaped.
struct Escaped<'a>(&'a str);

impl fmt::Display for Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.chars() {
            match ch {
                '&' => f.write_str("&amp;")?,
                '<' => f.write_str("&lt;")?,
                '>' => f.write_str("&gt;")?,
                _ => f.write_char(ch)?,
            }
        }
        Ok(())
    }
}
