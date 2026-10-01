/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Terminal state awareness module
//!
//! Provides tools for understanding the current state of a terminal:
//! - ANSI escape sequence parsing
//! - Virtual screen buffer simulation
//! - Terminal state queries (cursor, lines, prompts)
//! - Menu detection

mod ansi;
mod screen;
mod state;
mod style;
mod svg;

pub use ansi::AnsiParser;
pub use ansi::AnsiSequence;
pub use screen::Cell;
pub use screen::ScreenBuffer;
pub use state::Menu;
pub use state::TerminalState;
pub use style::Color;
pub use style::Style;
