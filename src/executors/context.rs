use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

use derive_builder::Builder;

use crate::newline::replace_crlf;

/// Context that describes the environment in which one or multiple [`super::execution::Execution`]s run in
#[derive(Clone, Default, Debug, PartialEq, Eq, Builder)]
pub struct Context {
    /// Optional timeout that limits the maximum execution length (can be
    /// overridden per-[`Execution`] in some [`super::executor::Executor`]s)
    #[builder(default)]
    pub timeout: Option<Duration>,

    /// Optional cwd path for the execution
    #[builder(default)]
    pub work_directory: Option<PathBuf>,

    /// Optional path for that holds temporary files
    #[builder(default)]
    pub temp_directory: Option<PathBuf>,

    /// Whether to combine STDOUT and STDERR into one stream
    #[builder(default)]
    pub combine_output: bool,

    /// Whether CRLF and LF are all considered LF or not
    #[builder(default)]
    pub crlf_support: bool,
}

impl Context {
    /// Create a new Execution only from a shell expression, with empty
    /// environment and no timeout
    pub fn new() -> Self {
        Default::default()
    }

    /// Return clone of self with given timeout
    pub fn with_timeout(&self, timeout: Option<Duration>) -> Self {
        let mut clone = self.clone();
        clone.timeout = timeout;
        clone
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
    use super::ContextBuilder;
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
            let context = ContextBuilder::default()
                .crlf_support(*crlf_support)
                .build()
                .expect("create execution context");
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
