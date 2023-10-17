use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

/// Configuration for the scope of a whole document, that may contain multiple testcases
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct DocumentConfig {
    /// Include these paths in order, as if they were part of this file. All tests
    /// within the appended paths are appended to the tests defined in this file.
    /// Use-case is common/shared test tear-down. Paths must be relative to the
    /// current `$TESTDIR`
    pub append: Vec<PathBuf>,

    /// Defaults for per-test configurations
    pub defaults: TestCaseConfig,

    /// List of code block annotation languages that are Scrut should consider
    /// test cases.
    pub language_markers: Vec<String>,

    /// Include these paths in order, as if they were part of this file. All tests
    /// within the prepend paths are prepended to the tests defined in this file.
    /// Use-case is common/shared test setup. Paths must be relative to the
    /// current `$TESTDIR`
    pub prepend: Vec<PathBuf>,

    /// The path to the shell. If a full path is not provided, then the command
    /// must be in $PATH.
    pub shell: Option<PathBuf>,

    /// Timeout for the executions of all tests.
    pub total_timeout: Option<Duration>,
}

/// Controls which output streams are being considered when comparing to tests
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum OutputStreamControl {
    /// Consider only STDOUT when evaluating expectations
    Stdout,
    /// Consider only STDERR when evaluating expectations
    Stderr,
    /// Consider both STDOUT and STDERR when evaluating expectations
    /// Caution: Order of STDOUT and STDERR is not guaranteed.
    Combined,
    // Leave it to the user to explicitly mark which output expectations are for
    // STDOUT and which are for STDERR by adding `@STDOUT` and `@STDERR` marks
    // that denote that all following expectations (until the next mar or the
    // end) are for the identified stream
    Marked,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestCaseWait {
    /// How long to wait for
    pub timeout: Duration,

    /// If set then the wait will end early once the path exists
    pub path: Option<PathBuf>,
}

/// Configuration for the scope of a single [`crate::testcase::TestCase`]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestCaseConfig {
    /// Tell Scrut that the shell expression of this test will detach itself, so
    /// Scrut will not consider this a test (i.e. no output or exit code evaluation).
    /// Purpose is to allow the user to detach a command (like
    /// `nohup some-command &`) that is doing something asynchronous (e.g.
    /// starting a server to which the tested CLI is a client).
    pub detached: Option<bool>,

    /// A set of environment variable names and values that will be explicitly set
    /// for the test.
    pub environment: HashMap<String, String>,

    /// Whether CRLF should be translated to LF (=false) or whether CR needs to
    /// be explicitly handled (=true).
    pub keep_crlf: Option<bool>,

    /// Which output stream to choose when applying output expectations:
    /// - `Stdout`: All expectations apply to what is printed on STDOUT
    /// - `Stderr`: All expectations apply to what is printed on STDERR
    /// - `Combined`: STDOUT and STDERR will combined into a single stream where all expectations are applied on
    /// - `Marked` (todo): User marks which expectations are intended for which stream explicitly
    pub output_stream: Option<OutputStreamControl>,

    /// The exit code, that if returned by any test, leads to skipping of the whole file.
    pub skip_code: Option<u32>,

    /// A max execution time a test can run before it is considered failed (and
    /// will be aborted).
    pub timeout: Option<Duration>,

    /// Sleep for some time before starting this test (i.e. continuing with testing).
    /// If path is provided, then wait will be aborted (and the testing continues)
    /// as soon as path exists and the test will fail if it does not show up
    /// within duration. The wait time does not count against timeout(), but
    /// against total_timeout(). To be used in conjunction with detached().
    pub wait: Option<TestCaseWait>,
}
