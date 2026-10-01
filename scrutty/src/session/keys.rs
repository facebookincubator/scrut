/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Key constants for TTY input
//!
//! Provides named constants for special keys that can be sent to a TTY session.

/// Named key constants for TTY input
///
/// These can be used with `TtySession::send_keys()` or `TtySession::send_key()`.
///
/// # Example
///
/// ```rust,ignore
/// session.send_key(Keys::ENTER)?;
/// session.send_keys(&[Keys::DOWN, Keys::DOWN, Keys::ENTER])?;
/// ```
pub struct Keys;

impl Keys {
    // Arrow keys
    pub const UP: &'static str = "\x1b[A";
    pub const DOWN: &'static str = "\x1b[B";
    pub const RIGHT: &'static str = "\x1b[C";
    pub const LEFT: &'static str = "\x1b[D";

    // Common keys
    pub const ENTER: &'static str = "\r";
    pub const SPACE: &'static str = " ";
    pub const TAB: &'static str = "\t";
    pub const ESCAPE: &'static str = "\x1b";
    pub const BACKSPACE: &'static str = "\x7f";
    pub const DELETE: &'static str = "\x1b[3~";

    // Control keys
    pub const CTRL_A: &'static str = "\x01";
    pub const CTRL_B: &'static str = "\x02";
    pub const CTRL_C: &'static str = "\x03";
    pub const CTRL_D: &'static str = "\x04";
    pub const CTRL_E: &'static str = "\x05";
    pub const CTRL_F: &'static str = "\x06";
    pub const CTRL_G: &'static str = "\x07";
    pub const CTRL_H: &'static str = "\x08";
    pub const CTRL_K: &'static str = "\x0b";
    pub const CTRL_L: &'static str = "\x0c";
    pub const CTRL_N: &'static str = "\x0e";
    pub const CTRL_O: &'static str = "\x0f";
    pub const CTRL_P: &'static str = "\x10";
    pub const CTRL_R: &'static str = "\x12";
    pub const CTRL_S: &'static str = "\x13";
    pub const CTRL_U: &'static str = "\x15";
    pub const CTRL_W: &'static str = "\x17";
    pub const CTRL_X: &'static str = "\x18";
    pub const CTRL_Y: &'static str = "\x19";
    pub const CTRL_Z: &'static str = "\x1a";

    // Function keys
    pub const F1: &'static str = "\x1bOP";
    pub const F2: &'static str = "\x1bOQ";
    pub const F3: &'static str = "\x1bOR";
    pub const F4: &'static str = "\x1bOS";
    pub const F5: &'static str = "\x1b[15~";
    pub const F6: &'static str = "\x1b[17~";
    pub const F7: &'static str = "\x1b[18~";
    pub const F8: &'static str = "\x1b[19~";
    pub const F9: &'static str = "\x1b[20~";
    pub const F10: &'static str = "\x1b[21~";
    pub const F11: &'static str = "\x1b[23~";
    pub const F12: &'static str = "\x1b[24~";

    // Navigation keys
    pub const HOME: &'static str = "\x1b[H";
    pub const END: &'static str = "\x1b[F";
    pub const PAGE_UP: &'static str = "\x1b[5~";
    pub const PAGE_DOWN: &'static str = "\x1b[6~";
    pub const INSERT: &'static str = "\x1b[2~";
}

/// Convert a human-readable key name to its escape sequence
///
/// Supports names like "UP", "DOWN", "ENTER", "CTRL+C", etc.
/// Returns the input unchanged if not a recognized key name.
pub fn key_name_to_sequence(name: &str) -> &str {
    match name {
        "UP" => Keys::UP,
        "DOWN" => Keys::DOWN,
        "LEFT" => Keys::LEFT,
        "RIGHT" => Keys::RIGHT,
        "ENTER" => Keys::ENTER,
        "SPACE" => Keys::SPACE,
        "TAB" => Keys::TAB,
        "ESCAPE" | "ESC" => Keys::ESCAPE,
        "BACKSPACE" => Keys::BACKSPACE,
        "DELETE" | "DEL" => Keys::DELETE,
        "CTRL+A" => Keys::CTRL_A,
        "CTRL+B" => Keys::CTRL_B,
        "CTRL+C" => Keys::CTRL_C,
        "CTRL+D" => Keys::CTRL_D,
        "CTRL+E" => Keys::CTRL_E,
        "CTRL+F" => Keys::CTRL_F,
        "CTRL+G" => Keys::CTRL_G,
        "CTRL+H" => Keys::CTRL_H,
        "CTRL+K" => Keys::CTRL_K,
        "CTRL+L" => Keys::CTRL_L,
        "CTRL+N" => Keys::CTRL_N,
        "CTRL+O" => Keys::CTRL_O,
        "CTRL+P" => Keys::CTRL_P,
        "CTRL+R" => Keys::CTRL_R,
        "CTRL+S" => Keys::CTRL_S,
        "CTRL+U" => Keys::CTRL_U,
        "CTRL+W" => Keys::CTRL_W,
        "CTRL+X" => Keys::CTRL_X,
        "CTRL+Y" => Keys::CTRL_Y,
        "CTRL+Z" => Keys::CTRL_Z,
        "F1" => Keys::F1,
        "F2" => Keys::F2,
        "F3" => Keys::F3,
        "F4" => Keys::F4,
        "F5" => Keys::F5,
        "F6" => Keys::F6,
        "F7" => Keys::F7,
        "F8" => Keys::F8,
        "F9" => Keys::F9,
        "F10" => Keys::F10,
        "F11" => Keys::F11,
        "F12" => Keys::F12,
        "HOME" => Keys::HOME,
        "END" => Keys::END,
        "PAGEUP" | "PAGE_UP" => Keys::PAGE_UP,
        "PAGEDOWN" | "PAGE_DOWN" => Keys::PAGE_DOWN,
        "INSERT" | "INS" => Keys::INSERT,
        other => other,
    }
}
