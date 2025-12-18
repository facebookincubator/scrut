/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Context;
use anyhow::Result;
use clap::ValueEnum;
use unicode_categories::UnicodeCategories;

use crate::newline::BytesNewline;

/// Provide ASCII and unicode compatible strings with all non-printable
/// characters escaped
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum Escaper {
    /// All non ASCII and all non-printable ASCII characters are escaped
    Ascii,
    /// All non-printable Unicode characters are escaped
    #[default]
    Unicode,
}

impl Escaper {
    /// Returns provided byte sequence as string where all non-printable characters
    /// are escaped
    pub fn escaped_printable(&self, raw: &[u8]) -> String {
        match self {
            Escaper::Ascii => escaped_printable_ascii(raw),
            Escaper::Unicode => escaped_printable_unicode(raw),
        }
    }

    /// Returns provided byte sequence as string where all non-printable characters
    /// are escaped with the ` (escaped)` expectation marker added, when appropriate
    pub fn escaped_expectation(&self, raw: &[u8]) -> String {
        match self {
            Escaper::Ascii => escaped_expectation_ascii(raw),
            Escaper::Unicode => escaped_expectation_unicode(raw),
        }
    }

    pub fn has_unprintable(&self, raw: &[u8]) -> bool {
        match self {
            Escaper::Ascii => has_unprintable_ascii(raw),
            Escaper::Unicode => has_unprintable_unicode(raw),
        }
    }
}

/// Convenience wrapper for [`String::from_utf8_lossy`]
#[macro_export]
macro_rules! lossy_string {
    ($arg:expr) => {{ String::from_utf8_lossy($arg).to_string() }};
}

/// All non-printable bytes are rendered as hexadecimal escape sequence, all
/// white-spaces as escaped character classes
fn escaped_printable_ascii(bytes: &[u8]) -> String {
    if has_unprintable_ascii(bytes) {
        bytes.iter().map(byte_to_ascii).collect()
    } else {
        String::from_utf8_lossy(bytes).into()
    }
}

/// Returns whether given byte sequence contains characters from the non-printable
/// ASCII range
fn has_unprintable_ascii(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| !matches!(*byte, b'\x20'..=b'\x7e'))
}

fn byte_to_ascii(byte: &u8) -> String {
    match byte {
        // printable, but whitespace
        b'\n' => "\\n".to_string(),
        b'\r' => "\\r".to_string(),
        b'\t' => "\\t".to_string(),

        // printable, control
        //b'\x00' => "\\0".to_string(), <-- cannot be supported, as it conflicts with octal notation like "\077"
        b'\x07' => "\\a".to_string(),
        b'\x08' => "\\b".to_string(),
        b'\x0c' => "\\f".to_string(),
        b'\x0b' => "\\v".to_string(),

        // backslashes must now be escaped themselves
        b'\\' => "\\\\".to_string(),

        // printable character
        b'\x20'..=b'\x7e' => (*byte as char).to_string(),

        // .. the rest is NOT printable
        _ => format!("\\x{:02x}", *byte),
    }
}

/// Renders given line either with escape sequences (if it contains non-printable
/// characters) and denoted as `(escaped)` - or as-is.
fn escaped_expectation_ascii(line: &[u8]) -> String {
    let escaped = escaped_printable_ascii(line.trim_newlines());
    let encoded = lossy_string!(line.trim_newlines());
    if encoded == escaped {
        encoded
    } else {
        format!("{escaped} (escaped)")
    }
}

/// All non-printable unicode are rendered as hexadecimal escape sequence, all
/// white-spaces as escaped character classes, everything else is printed
fn escaped_printable_unicode(bytes: &[u8]) -> String {
    let mut seq = [0; 4];
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s
            .chars()
            .map(|c| {
                if c.is_other() {
                    let raw = c.encode_utf8(&mut seq).as_bytes();
                    escaped_printable_ascii(raw)
                } else {
                    c.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("");
    }

    // fallback in case this is not really a unicode string, but a binary sequence
    escaped_printable_ascii(bytes)
}

/// Renders given line either with escape sequences (if it contains non-printable
/// characters) and denoted as `(escaped)` - or as-is.
fn escaped_expectation_unicode(line: &[u8]) -> String {
    let escaped = escaped_printable_unicode(line.trim_newlines());
    let encoded = lossy_string!(line.trim_newlines());
    if encoded == escaped {
        encoded
    } else {
        format!("{escaped} (escaped)")
    }
}

/// Returns whether given byte sequence contains unicode characters that are not
/// not printable
fn has_unprintable_unicode(bytes: &[u8]) -> bool {
    String::from_utf8(bytes.to_vec())
        .map(|s| s.chars().any(|c| c.is_other()))
        .unwrap_or(true)
}

pub fn strip_colors(input: &str) -> Result<String> {
    let stripped = strip_colors_bytes(input.as_bytes())?;
    String::from_utf8(stripped).context("decode stripped bytes back to utf8 string")
}

pub fn strip_colors_bytes(input: &[u8]) -> Result<Vec<u8>> {
    strip_ansi_escapes::strip(input).context("strip ansi escape sequences from rendered output")
}

#[cfg(test)]
mod tests {

    use super::escaped_printable_ascii;
    use super::escaped_printable_unicode;

    #[test]
    fn test_bytes_as_printable_unicode() {
        let tests = vec![
            ("foo", "foo"),
            ("foo \x1b[1mbar\x1b[0m", "foo \\x1b[1mbar\\x1b[0m"),
            ("foo 😂", "foo 😂"),
            ("f🦀o", "f🦀o"),
            ("foo\tbar", "foo\\tbar"),
            ("foo \x1b[1m\t😂\x1d[0m", "foo \\x1b[1m\\t😂\\x1d[0m"),
            (
                "foo \x1b[2;5;0;31;47m\t😂\x1d[0m",
                "foo \\x1b[2;5;0;31;47m\\t😂\\x1d[0m",
            ),
        ];

        for (from, expect) in tests {
            let escaped = escaped_printable_unicode(from.as_bytes());
            assert_eq!(*expect, escaped, "from `{from}`");
        }
    }

    #[test]
    fn test_bytes_as_printable_ascii() {
        let tests = vec![
            ("\x00\x01\x02", "\\x00\\x01\\x02"),
            ("foo", "foo"),
            ("foo \x1b[1mbar\x1b[0m", "foo \\x1b[1mbar\\x1b[0m"),
            ("foo 😂", "foo \\xf0\\x9f\\x98\\x82"),
            ("foo\tbar", "foo\\tbar"),
            (
                "foo \x1b[1m\t😂\x1d[0m",
                "foo \\x1b[1m\\t\\xf0\\x9f\\x98\\x82\\x1d[0m",
            ),
            (
                "foo \x1b[2;5;0;31;47m\t😂\x1d[0m",
                "foo \\x1b[2;5;0;31;47m\\t\\xf0\\x9f\\x98\\x82\\x1d[0m",
            ),
        ];

        for (from, expect) in tests {
            let escaped = escaped_printable_ascii(from.as_bytes());
            assert_eq!(expect, &escaped, "from `{from}`");
        }
    }
}
