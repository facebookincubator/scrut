/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;

/// Some rule expressions require additionally escaping, e.g. a glob expression
/// that contains non-printable characters. In this case, the glob expression
/// is additionally annotated with a tailing (variant of) `(escaped)`.
/// This function returns the expression without that tailing annotation, so
/// [`apply_escaped_filter_bytes`] can be applied, or nothing if there is no
/// annotation
pub(super) fn expression_as_escaped(expression: &str) -> Option<&str> {
    for sequence in [" (escaped)", " \\(escaped\\)", " (esc)", " \\(esc\\)"] {
        if expression.ends_with(sequence) {
            return Some(&expression[0..(expression.len() - sequence.len())]);
        }
    }
    None
}

/// Applied to expressions that are escaped (i.e. end in ` (esc)` or ` (escaped)`)
/// returns the raw bytes representing the string, in which all escape sequences
/// (and double doubled tab characters) are resolved.
pub(super) fn apply_escaped_filter_bytes(expression: &str) -> Result<Vec<u8>> {
    let expression = unescape_tabs(expression);
    let expression = resolve_escape_sequences_to_bytes(&expression)
        .context("resolve escape sequences from escaped expression")?;
    Ok(expression)
}

/// Same as [`apply_escaped_filter_bytes`], but also decodes resolves bytes
/// (back) into UTF-8 string
pub(super) fn apply_escaped_filter_utf8(expression: &str) -> Result<String> {
    let expression = apply_escaped_filter_bytes(expression)?;
    let expression = String::from_utf8(expression).context("encode escaped expression to utf-8")?;
    Ok(expression)
}

/// Returns byte slice in which all hexadecimal or octal escape sequences
/// (`\\x0F` or `\\007`) are resolved into the respective bytes
fn resolve_escape_sequences_to_bytes(escaped: &str) -> Result<Vec<u8>> {
    let mut chars = escaped.chars();
    let mut bytes = vec![];

    macro_rules! collect_sequence {
        () => {{
            let mut sequence = vec![];
            sequence.push(
                chars
                    .next()
                    .ok_or_else(|| anyhow!("missing first character in escape sequence"))?,
            );
            sequence.push(
                chars
                    .next()
                    .ok_or_else(|| anyhow!("missing second character in escape sequence"))?,
            );
            let sequence: String = sequence.into_iter().collect();
            sequence
        }};
    }

    let mut buf: [u8; 4] = [0; 4];
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(ch2) = chars.next() {
                    match ch2 {
                        '0' => {
                            let octal = collect_sequence!();
                            bytes.push(
                                u8::from_str_radix(&octal, 8)
                                    .with_context(|| format!("octal from `{octal}`"))?,
                            );
                        }
                        'x' => {
                            let hex = collect_sequence!();
                            bytes.push(
                                u8::from_str_radix(&hex, 16)
                                    .with_context(|| format!("hex from `{hex}`"))?,
                            );
                        }
                        '\\' => bytes.push(ch as u8),
                        _ => {
                            bytes.push(ch as u8);
                            bytes.push(ch2 as u8);
                        }
                    }
                } else {
                    // TBD: keep or be laissez-faire about it?
                    bail!("unused tailing escape")
                }
            }
            _ => {
                let add = ch.encode_utf8(&mut buf).as_bytes();
                bytes.extend(add)
            }
        }
    }
    Ok(bytes)
}

/// Resolves all escaped `\\t` sequences into `\t` characters
pub(crate) fn unescape_tabs(escaped: &str) -> String {
    let mut chars = escaped.chars();
    let mut out = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(ch2) = chars.next() {
                match ch2 {
                    // All ASCII control characters: https://en.wikipedia.org/wiki/Control_character#In_ASCII
                    //'0' => out.push('\x00'), <-- cannot be supported, as it conflicts with octal notation like "\077"
                    'a' => out.push('\x07'),
                    'b' => out.push('\x08'),
                    'e' => out.push('\x1b'),
                    'f' => out.push('\x0c'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'v' => out.push('\x0b'),
                    any => {
                        out.push('\\');
                        out.push(any);
                    }
                }
                continue;
            }
        }
        out.push(ch)
    }
    out
}

#[cfg(test)]
mod tests {

    use super::expression_as_escaped;
    use super::resolve_escape_sequences_to_bytes;
    use super::unescape_tabs;

    #[test]
    fn test_expression_as_escaped() {
        let tests = vec![
            ("foo", None),
            ("foo (escaped)", Some("foo")),
            ("foo (esc)", Some("foo")),
            ("foo (escaped) not", None),
        ];
        for (from, expect) in tests {
            let updated = expression_as_escaped(from);
            assert_eq!(expect, updated, "from `{}`", from);
        }
    }

    #[test]
    fn test_resolve_escape_sequences_to_bytes() {
        let tests = vec![
            ("foo", b"foo".to_vec()),
            ("foo\\x12bar", b"foo\x12bar".to_vec()),
            ("foo\nbar", b"foo\nbar".to_vec()),
            ("foo\\nbar", b"foo\\nbar".to_vec()),
            ("foo\\\nbar", b"foo\\\nbar".to_vec()),
            ("foo\\\\nbar", b"foo\\nbar".to_vec()),
            ("\\\\", b"\\".to_vec()),
            ("\\\\\\\\", b"\\\\".to_vec()),
        ];
        for (i, (from, expect)) in tests.iter().enumerate() {
            let to = resolve_escape_sequences_to_bytes(from).expect("resolves");
            //crate::debug_bytewise!(&format!("`{:02} from `{}` ", i + 1, from), expect, &to);
            assert_eq!(expect, &to, "{:02} from `{}`", i + 1, from);
        }
    }

    #[test]
    fn test_unescape_tabs() {
        let tests = vec![
            ("foo", "foo"),
            ("foo\tbar", "foo\tbar"),
            ("foo\\nbar", "foo\\nbar"),
            ("foo\\tbar", "foo\tbar"),
        ];
        for (from, expect) in tests {
            let to = unescape_tabs(from);
            assert_eq!(expect, &to, "from `{}`", from);
        }
    }
}
