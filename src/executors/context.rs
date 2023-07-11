use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crate::newline::replace_crlf;

/// Context that describes the environment in which one or multiple [`super::execution::Execution`]s run in
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Context {
    /// Optional timeout that limits the maximum execution length (can be
    /// overridden per-[`Execution`] in some [`super::executor::Executor`]s)
    pub timeout: Option<Duration>,

    /// Optional cwd path for the execution
    pub directory: Option<PathBuf>,

    /// Whether to combine STDOUT and STDERR into one stream
    pub combine_output: bool,

    /// Whether CRLF and LF are all considered LF or not
    pub crlf_support: bool,
}

impl Context {
    /// Create a new Execution only from a shell expression, with empty
    /// environment and no timeout
    pub fn new() -> Self {
        Default::default()
    }

    /// Builder setter for timeout
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Builder setter for working directory
    pub fn directory(mut self, directory: &Path) -> Self {
        self.directory = Some(directory.to_path_buf());
        self
    }

    /// Builder setter for whether to combine STDOUT and STDERR
    pub fn combine_output(mut self, combined_output: bool) -> Self {
        self.combine_output = combined_output;
        self
    }

    /// Builder setter for whether to support CRLF in outputs
    pub fn crlf_support(mut self, crlf_support: bool) -> Self {
        self.crlf_support = crlf_support;
        self
    }

    /// Render provided output lines based on CRLF setting
    pub fn render_output<'a>(&self, output: &'a [u8]) -> Cow<'a, [u8]> {
        if self.crlf_support {
            Cow::from(output)
        } else {
            replace_crlf(output)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::lossy_string;

    #[test]
    fn test_context_render_output() {
        let tests = &[
            (false, "foo", "foo"),
            (true, "foo", "foo"),
            (false, "foo\nbar\nbaz", "foo\nbar\nbaz"),
            (true, "foo\nbar\nbaz", "foo\nbar\nbaz"),
            (false, "foo\r\nbar\r\nbaz", "foo\nbar\nbaz"),
            (true, "foo\r\nbar\r\nbaz", "foo\r\nbar\r\nbaz"),
        ];
        for (crlf_support, from, expect) in tests {
            let context = Context::new().crlf_support(*crlf_support).to_owned();
            let output = context.render_output(from.as_bytes());
            assert_eq!(
                *expect,
                lossy_string!(&output),
                "from {} (crlf = {})",
                *from,
                *crlf_support
            );
        }
    }
}
