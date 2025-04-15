/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! This module provides convenient handlers for handling common newline relating
//! operations, like assuring that strings do end (or do not end) in a newline.
//! Also, since Scrut deals with raw byte output of scripts, that may not even
//! be unicode, the whole shabang is implemented for byte arrays as well.
//!
//! Regarding CRLF and LF line endings:
//! Scrut internally only works with LF endings and considers CRLF only at
//! certain boundaries, which are:
//!
//! - Content read from test files: CRLF is converted into LF
//! - Output generated from test execution: CRLF is converter into LF
//!
//! In line with the Rust policy, Scrut does _NOT_ output CRLF at any time.
use std::borrow::Cow;

/// Extends byte slices to provide easy interface dealing with newline characters
pub(crate) trait BytesNewline {
    /// Return the byte slice without tailing newline character(s)
    fn trim_newlines(&self) -> &[u8];

    /// Returns byte slice that ends in newline character(s)
    fn assure_newline(&self) -> Cow<'_, [u8]>;

    /// Return bool whether ends in new line
    fn ends_in_newline(&self) -> bool;
}

impl BytesNewline for &[u8] {
    fn trim_newlines(&self) -> &[u8] {
        let mut right_index = self.len();
        while ends_in_newline(&self[0..right_index]) {
            right_index -= 1;
        }
        &self[0..right_index]
    }

    fn assure_newline(&self) -> Cow<'_, [u8]> {
        if ends_in_newline(self) {
            return (*self).into();
        }
        let mut updated = self.to_vec();
        updated.push(b'\n');
        updated.into()
    }

    fn ends_in_newline(&self) -> bool {
        ends_in_newline(self)
    }
}

fn ends_in_newline(data: &[u8]) -> bool {
    let l = data.len();
    l > 0 && data[l - 1] == b'\n'
}

/// Extend strings with interface to assure
pub(crate) trait StringNewline {
    fn trim_newlines(&self) -> Cow<'_, str>;
    fn assure_newline(&self) -> Cow<'_, str>;
}

impl StringNewline for &str {
    fn assure_newline(&self) -> Cow<'_, str> {
        assure_newline(self)
    }

    fn trim_newlines(&self) -> Cow<'_, str> {
        trim_newlines(self)
    }
}

impl StringNewline for String {
    fn assure_newline(&self) -> Cow<'_, str> {
        assure_newline(self)
    }

    fn trim_newlines(&self) -> Cow<'_, str> {
        trim_newlines(self)
    }
}

fn assure_newline(line: &str) -> Cow<'_, str> {
    if line.ends_with('\n') {
        line.into()
    } else {
        (line.to_string() + "\n").into()
    }
}

fn trim_newlines(line: &'_ str) -> Cow<'_, str> {
    if let Some(stripped) = line.strip_suffix('\n') {
        trim_newlines(stripped)
    } else {
        line.into()
    }
}

/// A helper trait that provides a convenient way to split byte arrays into lines
pub(crate) trait SplitLinesByNewline {
    /// Splits byte array by LF while keeping the ending LF for each item
    /// of the returned vector
    fn split_at_newline(&self) -> Vec<&[u8]>;
}

impl SplitLinesByNewline for &[u8] {
    fn split_at_newline(&self) -> Vec<&[u8]> {
        split_at_newline(self)
    }
}

fn split_at_newline(text: &[u8]) -> Vec<&[u8]> {
    let mut lines = vec![];
    let mut start = 0;

    for (index, byte) in text.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(&text[start..index + 1]);
            start = index + 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..])
    }

    lines
}

const CRLF: &[u8] = b"\r\n";

/// Replaces all CRLF with LF
pub fn replace_crlf<'a>(bytes: &'a [u8]) -> Cow<'a, [u8]> {
    if let Some(index) = bytes.windows(2).position(|window| window == CRLF) {
        [
            Cow::from(&bytes[0..index]),
            replace_crlf(&bytes[index + 1..]),
        ]
        .concat()
        .into()
    } else {
        bytes.into()
    }
}

/// Like the [`format`] with an added new line character
#[macro_export]
macro_rules! formatln {
    ($arg:expr) => {{
        format!("{}\n", $arg)
    }};
    ($arg:tt, $($args:tt)*) => {{
        format!("{}\n", format!($arg, $($args)*))
    }};
}

/// Like the [`format`] with an added new line character and in raw bytes
#[macro_export]
macro_rules! bformatln {
    ($arg:expr) => {{
        format!("{}\n", $arg).as_bytes().to_vec()
    }};
    ($arg:tt, $($args:tt)*) => {{
        format!("{}\n", format!($arg, $($args)*)).as_bytes().to_vec()
    }};
}

#[macro_export]
macro_rules! blines {
    ($arg:tt) => {{
        format!("{}\n", $arg).as_bytes().to_vec()
    }};
    ($arg:tt, $($args:tt)*) => {{
        ([$arg, $($args)*].join("\n") + "\n").as_bytes().to_vec()
    }};
}

#[cfg(test)]
mod tests {
    use super::BytesNewline;
    use super::SplitLinesByNewline;
    use super::StringNewline;
    use super::replace_crlf;
    use crate::newline::assure_newline;

    #[test]
    fn test_formatln() {
        assert_eq!(formatln!("something"), "something\n");
        assert_eq!(
            formatln!("something {}", "bla"),
            "something bla\n".to_string()
        );
        assert_eq!(
            formatln!("something {} {}", "bla", "foo"),
            "something bla foo\n".to_string(),
        );
    }

    #[test]
    fn test_bformatln() {
        assert_eq!(bformatln!("something"), "something\n".as_bytes());
        assert_eq!(
            bformatln!("something {}", "bla"),
            "something bla\n".as_bytes(),
        );
        assert_eq!(
            bformatln!("something {} {}", "bla", "foo"),
            "something bla foo\n".as_bytes()
        );
    }

    #[test]
    fn test_blines() {
        assert_eq!(blines!("something"), "something\n".as_bytes(),);
        assert_eq!(blines!("something", "bla"), "something\nbla\n".as_bytes());
        assert_eq!(
            blines!("something", "bla", "foo"),
            "something\nbla\nfoo\n".as_bytes()
        );
    }

    #[test]
    fn test_trim_newlines_bytes() {
        let tests = vec![("foo", "foo"), ("foo\n", "foo"), ("foo\n\n\n", "foo")];

        for (from, expect) in tests {
            let fromb = from.as_bytes();
            let to = fromb.trim_newlines();
            assert_eq!(expect.as_bytes(), to, "from `{:?}`", fromb);
        }
    }

    #[test]
    fn test_assure_newline_bytes() {
        let tests = vec![
            ("foo", "foo\n"),
            ("foo\n", "foo\n"),
            ("foo\n\n\n", "foo\n\n\n"),
        ];

        for (from, expect) in tests {
            let fromb = from.as_bytes();
            let to = fromb.assure_newline().to_vec();
            assert_eq!(expect.as_bytes().to_vec(), to, "from `{:?}`", fromb);
        }
    }

    #[test]
    fn test_trim_newlines_string() {
        let tests = vec![("foo", "foo"), ("foo\n", "foo"), ("foo\n\n\n", "foo")];
        for (from, expect) in tests {
            let result = from.trim_newlines();
            assert_eq!(expect, result, "from {}", from);
        }
    }

    #[test]
    fn test_assure_newline_string() {
        let tests = vec![
            ("foo", "foo\n"),
            ("foo\n", "foo\n"),
            ("foo\n\n\n", "foo\n\n\n"),
        ];
        for (from, expect) in tests {
            let result = assure_newline(from);
            assert_eq!(expect, &result, "from {}", from);
        }
    }

    #[test]
    fn test_split_at_newline() {
        let tests = vec![
            ("l1", vec![b"l1" as &[u8]]),
            ("l1\n", vec![b"l1\n"]),
            ("l1\nl2", vec![b"l1\n", b"l2"]),
            ("l1\nl2\nl3\n", vec![b"l1\n", b"l2\n", b"l3\n"]),
        ];
        for (from, expect) in tests {
            let to = from.as_bytes();
            let to = to.split_at_newline();
            assert_eq!(expect, to, "from `{}`", from)
        }
    }

    #[test]
    fn test_replace_crlf() {
        let tests = vec![
            ("l1", "l1"),
            ("l1\nl2", "l1\nl2"),
            ("l1\r\nl2", "l1\nl2"),
            ("l1\r\nl2\r\n", "l1\nl2\n"),
            ("\r\nl1\r\nl2\r\n", "\nl1\nl2\n"),
        ];

        for (from, expect) in tests {
            let to = replace_crlf(from.as_bytes());
            assert_eq!(expect.as_bytes().to_vec(), to.to_vec(),);
        }
    }
}
