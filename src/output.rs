/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Debug;
use std::fmt::Display;
use std::time::Duration;

use serde::ser::SerializeMap;
use serde::Serialize;

use crate::config::DEFAULT_SKIP_DOCUMENT_CODE;
use crate::escaping::Escaper;
use crate::formatln;
use crate::lossy_string;
use crate::newline::SplitLinesByNewline;

/// Product of a single execution that captures output and status
#[derive(Clone, PartialEq, Eq)]
pub struct Output {
    /// The STDERR output of the execution
    pub stderr: OutputStream,

    /// The STDOUT output of the execution
    pub stdout: OutputStream,

    /// The exit code the execution ended in. A value of `None` implies the
    /// execution did not return (i.e. aborted due to timeout)
    pub exit_code: ExitStatus,
}

impl Output {
    pub fn to_error_string(&self, escaper: &Escaper) -> String {
        let mut err = String::new();
        err.push_str(&formatln!("## STDOUT"));
        err.push_str(&self.stdout.to_output_string(Some("#> "), escaper));
        err.push_str(&formatln!("## STDERR"));
        err.push_str(&self.stderr.to_output_string(Some("#> "), escaper));
        err
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let escaper = &Escaper::default();
        let stdout = escaper
            .escaped_printable(&self.stdout.0)
            .replace("\\n", "\\n\n");
        let stderr = escaper
            .escaped_printable(&self.stderr.0)
            .replace("\\n", "\\n\n");

        write!(
            f,
            "# STDOUT\n{}\n# STDERR\n{}\n# EXITCODE: {}\n",
            stdout,
            stderr,
            &self.exit_code.to_string(),
        )
    }
}

impl Default for Output {
    fn default() -> Self {
        Self {
            stdout: vec![].into(),
            stderr: vec![].into(),
            exit_code: ExitStatus::Unknown,
        }
    }
}

impl Serialize for Output {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("exit_code", &self.exit_code.to_string())?;
        map.serialize_entry("stdout", &lossy_string!((&self.stdout).into()))?;
        map.serialize_entry("stderr", &lossy_string!((&self.stderr).into()))?;
        map.end()
    }
}

impl<T: ToString, U: ToString> From<(T, U, Option<i32>)> for Output {
    fn from(set: (T, U, Option<i32>)) -> Self {
        Self {
            stdout: OutputStream(set.0.to_string().into()),
            stderr: OutputStream(set.1.to_string().into()),
            exit_code: match set.2 {
                None => ExitStatus::Unknown,
                Some(code) => ExitStatus::Code(code),
            },
        }
    }
}

impl<T: ToString, U: ToString> From<(T, U)> for Output {
    fn from(set: (T, U)) -> Self {
        Output::from((set.0, set.1, Some(0)))
    }
}

impl From<Duration> for Output {
    fn from(timeout: Duration) -> Self {
        Self {
            stdout: vec![].into(),
            stderr: vec![].into(),
            exit_code: ExitStatus::Timeout(timeout),
        }
    }
}

impl From<ExitStatus> for Output {
    fn from(status: ExitStatus) -> Self {
        Self {
            stdout: vec![].into(),
            stderr: vec![].into(),
            exit_code: status,
        }
    }
}

/// The status an execution can finish in
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitStatus {
    /// Execution resulted in exit code
    Code(i32),

    /// Execution never finished due to timeout
    Timeout(Duration),

    /// Execution was skipped
    Skipped,

    /// Implies the execution was intentionally detached and any output should
    /// be ignored.
    Detached,

    /// Execution failed for unknown reason
    Unknown,
}

impl ExitStatus {
    /// Exit code 0 denotes success
    pub const SUCCESS: Self = Self::Code(0);

    /// Returns exit code as integer with -1 for timeout and -255 for unknown
    pub fn as_code(&self) -> i32 {
        match self {
            Self::Code(code) => *code,
            Self::Skipped => DEFAULT_SKIP_DOCUMENT_CODE,
            Self::Timeout(_) => -1,
            Self::Detached => -100,
            Self::Unknown => -255,
        }
    }
}

impl Display for ExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code(code) => write!(f, "{}", code),
            Self::Timeout(duration) => write!(f, "timeout[{:.2}ms]", duration.as_millis()),
            Self::Skipped => write!(f, "skipped"),
            Self::Detached => write!(f, "detached"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<i32> for ExitStatus {
    fn from(value: i32) -> Self {
        ExitStatus::Code(value)
    }
}

impl From<ExitStatus> for i32 {
    fn from(value: ExitStatus) -> Self {
        value.as_code()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct OutputStream(Vec<u8>);

impl OutputStream {
    pub fn to_output_string(&self, prefix: Option<&str>, escaper: &Escaper) -> String {
        let prefix = prefix.unwrap_or("");
        let mut out = String::new();
        let bytes: &[u8] = self.into();
        let lines = bytes.split_at_newline();
        let ends_in_newline = !bytes.is_empty() && bytes[bytes.len() - 1] == b'\n';
        for (idx, line) in lines.iter().enumerate() {
            let expectation = escaper.escaped_expectation(line);
            let suffix = if !ends_in_newline
                && !expectation.ends_with(" (escaped)")
                && idx + 1 == lines.len()
            {
                " (no-eol)"
            } else {
                ""
            };
            out.push_str(&formatln!("{}{}{}", prefix, &expectation, suffix))
        }
        out
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<Vec<u8>> for OutputStream {
    fn from(stream: Vec<u8>) -> Self {
        Self(stream)
    }
}

impl From<&[u8]> for OutputStream {
    fn from(stream: &[u8]) -> Self {
        Self(stream.to_vec())
    }
}

impl From<&str> for OutputStream {
    fn from(stream: &str) -> Self {
        Self(stream.as_bytes().to_vec())
    }
}

impl From<&OutputStream> for Vec<u8> {
    fn from(stream: &OutputStream) -> Self {
        stream.0.clone()
    }
}

impl<'a> From<&'a OutputStream> for &'a [u8] {
    fn from(stream: &'a OutputStream) -> Self {
        &stream.0[..]
    }
}

#[cfg(test)]
mod tests {
    use super::OutputStream;
    use crate::escaping::Escaper;

    #[test]
    fn test_output_stream_appends_no_eol() {
        let tests = vec![
            ("a", "a (no-eol)\n"),
            ("a\n", "a\n"),
            ("a\nb", "a\nb (no-eol)\n"),
            ("a\nb\n", "a\nb\n"),
        ];
        for (from, expect) in tests {
            let stream = OutputStream(from.as_bytes().to_vec());
            let to = stream.to_output_string(None, &Escaper::default());
            assert_eq!(expect, &to, "from input '{from}'");
        }
    }

    #[test]
    fn test_prefixed_output_stream() {
        let tests = vec![
            ("a\n", "> a\n"),
            ("a\nb\n", "> a\n> b\n"),
            ("a\nb\nc\n", "> a\n> b\n> c\n"),
        ];
        for (from, expect) in tests {
            let stream = OutputStream(from.as_bytes().to_vec());
            let to = stream.to_output_string(Some("> "), &Escaper::default());
            assert_eq!(expect, &to, "from input '{from}'");
        }
    }
}
