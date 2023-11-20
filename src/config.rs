use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Duration;

use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// The default total (per-document) timeout in seconds
pub const DEFAULT_DOCUMENT_TIMEOUT: u64 = 900;

/// The exit code that any test execution can return to skip all tests in one document
pub const DEFAULT_SKIP_DOCUMENT_CODE: i32 = 80;

/// Configuration for the scope of a whole document, that may contain multiple testcases
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct DocumentConfig {
    /// Include these paths in order, as if they were part of this file. All tests
    /// within the appended paths are appended to the tests defined in this file.
    /// Use-case is common/shared test tear-down. Paths must be relative to the
    /// current `$TESTDIR`
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub append: Vec<PathBuf>,

    /// Defaults for per-test configurations
    #[serde(skip_serializing_if = "TestCaseConfig::is_empty")]
    pub defaults: TestCaseConfig,

    /// Include these paths in order, as if they were part of this file. All tests
    /// within the prepend paths are prepended to the tests defined in this file.
    /// Use-case is common/shared test setup. Paths must be relative to the
    /// current `$TESTDIR`
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub prepend: Vec<PathBuf>,

    /// The path to the shell. If a full path is not provided, then the command
    /// must be in $PATH.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<PathBuf>,

    /// Timeout for the executions of all tests.
    #[serde(
        skip_serializing_if = "is_none_or_default_timeout",
        deserialize_with = "parse_duration_opt",
        serialize_with = "render_duration_opt"
    )]
    pub total_timeout: Option<Duration>,
}

impl DocumentConfig {
    /// Returns instance with all values set to [`None`]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns default testcase configuration for Markdown documents
    ///
    /// Currently:
    /// - Total Timeout: 900s
    pub fn default_markdown() -> Self {
        Self {
            total_timeout: Some(Duration::from_secs(DEFAULT_DOCUMENT_TIMEOUT)),
            ..Default::default()
        }
    }

    /// Returns default testcase configuration for Cram documents
    ///
    /// Currently:
    /// - Total Timeout: 900s
    pub fn default_cram() -> Self {
        Self {
            total_timeout: Some(Duration::from_secs(DEFAULT_DOCUMENT_TIMEOUT)),
            ..Default::default()
        }
    }

    /// Returns true if none the configuration parameters are set
    pub fn is_empty(&self) -> bool {
        self.shell.is_none()
            && self.total_timeout.is_none()
            && self.prepend.is_empty()
            && self.append.is_empty()
            && self.defaults.is_empty()
    }

    /// Returns a new instance that fills in unset values from the provided defaults.
    /// Values for `append` and `prepend` are extended, not overwritten.
    pub fn with_defaults_from(&self, defaults: &Self) -> Self {
        // append is added at the end ..
        let mut append = defaults.append.clone();
        append.extend(self.append.clone());

        // prepend is added at the start
        let mut prepend = self.prepend.clone();
        prepend.extend(defaults.prepend.clone());

        Self {
            append,
            prepend,
            defaults: self.defaults.with_defaults_from(&defaults.defaults),
            shell: self.shell.clone().or_else(|| defaults.shell.clone()),
            total_timeout: self.total_timeout.or(defaults.total_timeout),
        }
    }

    /// Returns a new instance that is overridden with provided (set) values.
    /// Values for `append` and `prepend` are extended, not overwritten.
    pub fn with_overrides_from(&self, overrides: &Self) -> Self {
        overrides.with_defaults_from(self)
    }
}

impl Display for DocumentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = serde_json::to_string(&self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", out)
    }
}

fn is_none_or_default_timeout(timeout: &Option<Duration>) -> bool {
    if let Some(timeout) = timeout {
        timeout.as_secs() == DEFAULT_DOCUMENT_TIMEOUT
    } else {
        false
    }
}

/// Controls which output streams are being considered when comparing to tests
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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
    // TODO(implement) Marked,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestCaseWait {
    /// How long to wait for
    #[serde(
        deserialize_with = "parse_duration",
        serialize_with = "render_duration"
    )]
    pub timeout: Duration,

    /// If set then the wait will end early once the path exists
    pub path: Option<PathBuf>,
}

impl TestCaseWait {
    /// Deserialize from either scalar (only timeout) or map
    fn parse<'de, D>(deserializer: D) -> Result<Option<TestCaseWait>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TestCaseWaitParser(PhantomData<fn() -> Option<TestCaseWait>>);

        impl<'de> Visitor<'de> for TestCaseWaitParser {
            type Value = Option<TestCaseWait>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Option<TestCaseWait>, E>
            where
                E: de::Error,
            {
                let timeout = humantime::parse_duration(value).map_err(de::Error::custom)?;
                Ok(Some(TestCaseWait {
                    timeout,
                    path: None,
                }))
            }

            fn visit_map<M>(self, map: M) -> Result<Option<TestCaseWait>, M::Error>
            where
                M: MapAccess<'de>,
            {
                let wait = TestCaseWait::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(Some(wait))
            }
        }

        deserializer.deserialize_any(TestCaseWaitParser(PhantomData))
    }
}

/// Configuration for the scope of a single [`crate::testcase::TestCase`]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct TestCaseConfig {
    /// Tell Scrut that the shell expression of this test will detach itself, so
    /// Scrut will not consider this a test (i.e. no output or exit code evaluation).
    /// Purpose is to allow the user to detach a command (like
    /// `nohup some-command &`) that is doing something asynchronous (e.g.
    /// starting a server to which the tested CLI is a client).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detached: Option<bool>,

    /// A set of environment variable names and values that will be explicitly set
    /// for the test.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,

    /// Whether CRLF should be translated to LF (=false) or whether CR needs to
    /// be explicitly handled (=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_crlf: Option<bool>,

    /// Which output stream to choose when applying output expectations:
    /// - `Stdout`: All expectations apply to what is printed on STDOUT
    /// - `Stderr`: All expectations apply to what is printed on STDERR
    /// - `Combined`: STDOUT and STDERR will combined into a single stream where all expectations are applied on
    /// - `Marked` (todo): User marks which expectations are intended for which stream explicitly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_stream: Option<OutputStreamControl>,

    /// The exit code, that if returned by any test, leads to skipping of the whole file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_document_code: Option<i32>,

    /// A max execution time a test can run before it is considered failed (and
    /// will be aborted).
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "parse_duration_opt",
        serialize_with = "render_duration_opt"
    )]
    pub timeout: Option<Duration>,

    /// Sleep for some time before starting this test (i.e. continuing with testing).
    /// If path is provided, then wait will be aborted (and the testing continues)
    /// as soon as path exists and the test will fail if it does not show up
    /// within duration. The wait time does not count against timeout(), but
    /// against total_timeout(). To be used in conjunction with detached().
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "TestCaseWait::parse"
    )]
    pub wait: Option<TestCaseWait>,
}

impl TestCaseConfig {
    /// Returns instance with all values set to [`None`]
    pub fn empty() -> Self {
        Self::default() // TODO: review - default may soon not be empty anymore
    }

    /// Returns default testcase configuration for Markdown documents
    ///
    /// Currently:
    /// - Output Stream: STDOUT
    /// - Skip Code: 80
    ///
    /// TODO: change the default output stream to Combined (adjust all uses!)
    pub fn default_markdown() -> Self {
        Self {
            output_stream: Some(OutputStreamControl::Stdout),
            skip_document_code: Some(DEFAULT_SKIP_DOCUMENT_CODE),
            ..Default::default()
        }
    }

    /// Returns default testcase configuration for Cram documents
    ///
    /// Currently:
    /// - Output Stream: Combined STDOUT and STDERR
    /// - Skip Code: 80
    /// - Keep CRLF in output
    pub fn default_cram() -> Self {
        Self {
            output_stream: Some(OutputStreamControl::Combined),
            keep_crlf: Some(true),
            skip_document_code: Some(DEFAULT_SKIP_DOCUMENT_CODE),
            ..Default::default()
        }
    }

    /// Returns true if none the configuration parameters are set
    pub fn is_empty(&self) -> bool {
        self.output_stream.is_none()
            && self.keep_crlf.is_none()
            && self.timeout.is_none()
            && self.detached.is_none()
            && self.wait.is_none()
            && self.skip_document_code.is_none()
            && self.environment.is_empty()
    }

    /// Returns a new instance that fills in unset values from the provided defaults
    pub fn with_defaults_from(&self, defaults: &Self) -> Self {
        Self {
            output_stream: self
                .output_stream
                .clone()
                .or_else(|| defaults.output_stream.clone()),
            keep_crlf: self.keep_crlf.or(defaults.keep_crlf),
            timeout: self.timeout.or(defaults.timeout),
            environment: self
                .environment
                .clone()
                .into_iter()
                .chain(defaults.environment.clone())
                .collect(),
            detached: self.detached.or(defaults.detached),
            wait: self.wait.clone().or_else(|| defaults.wait.clone()),
            skip_document_code: self.skip_document_code.or(defaults.skip_document_code),
        }
    }

    /// Returns a new instance that is overridden with provided (set) values
    pub fn with_overrides_from(&self, overrides: &Self) -> Self {
        overrides.with_defaults_from(self)
    }

    /// Insert any environment variables that are not already defined (no overwrites)
    pub fn merge_environment(&self, environment: &BTreeMap<&str, &str>) -> Self {
        let mut config = self.clone();
        for (key, value) in environment {
            if !config.environment.contains_key(*key) {
                config.environment.insert((*key).into(), (*value).into());
            }
        }
        config
    }
}

impl Display for TestCaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = serde_json::to_string(&self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", out)
    }
}

fn parse_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    let duration = humantime::parse_duration(&value).map_err(de::Error::custom)?;
    Ok(duration)
}

fn parse_duration_opt<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    if value.is_empty() || value == "null" {
        return Ok(None);
    }
    let duration = humantime::parse_duration(&value).map_err(de::Error::custom)?;
    Ok(Some(duration))
}

fn render_duration<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = humantime::format_duration(*value).to_string();
    serializer.serialize_str(&value)
}

fn render_duration_opt<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = if let Some(value) = value {
        humantime::format_duration(*value).to_string()
    } else {
        "null".to_string()
    };
    serializer.serialize_str(&value)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::DocumentConfig;
    use super::TestCaseWait;
    use crate::config::OutputStreamControl;
    use crate::config::TestCaseConfig;

    const FULL_DOCUMENT_CONFIG: &str = "
append:
- app1
- app2
defaults:
  detached: true
  environment:
    BAZ: zoing
    FOO: bar
  keep_crlf: true
  output_stream: stdout
  skip_document_code: 123
  timeout: 6m 4s
  wait:
    timeout: 2m 1s
    path: the-wait-path
prepend:
- prep1
- prep2
shell: the-shell
total_timeout: 5m 3s
";

    #[test]
    fn test_parse_full_document_config() {
        let config: DocumentConfig =
            serde_yaml::from_str(FULL_DOCUMENT_CONFIG).expect("parse full document config");
        assert_eq!(
            config,
            DocumentConfig {
                shell: Some("the-shell".into()),
                total_timeout: Some(Duration::from_secs(5 * 60 + 3)),
                prepend: vec!["prep1".into(), "prep2".into()],
                append: vec!["app1".into(), "app2".into()],
                defaults: TestCaseConfig {
                    output_stream: Some(OutputStreamControl::Stdout),
                    keep_crlf: Some(true),
                    timeout: Some(Duration::from_secs(6 * 60 + 4)),
                    environment: {
                        let mut m = BTreeMap::new();
                        m.insert("FOO".to_string(), "bar".to_string());
                        m.insert("BAZ".to_string(), "zoing".to_string());
                        m
                    },
                    detached: Some(true),
                    wait: Some(TestCaseWait {
                        timeout: Duration::from_secs(2 * 60 + 1),
                        path: Some(PathBuf::from("the-wait-path")),
                    }),
                    skip_document_code: Some(123),
                }
            }
        )
    }

    #[test]
    fn test_render_full_document_config() {
        let config = DocumentConfig {
            shell: Some("the-shell".into()),
            total_timeout: Some(Duration::from_secs(5 * 60 + 3)),
            prepend: vec!["prep1".into(), "prep2".into()],
            append: vec!["app1".into(), "app2".into()],
            defaults: TestCaseConfig {
                output_stream: Some(OutputStreamControl::Stdout),
                keep_crlf: Some(true),
                timeout: Some(Duration::from_secs(6 * 60 + 4)),
                environment: {
                    let mut m = BTreeMap::new();
                    m.insert("FOO".to_string(), "bar".to_string());
                    m.insert("BAZ".to_string(), "zoing".to_string());
                    m
                },
                detached: Some(true),
                wait: Some(TestCaseWait {
                    timeout: Duration::from_secs(2 * 60 + 1),
                    path: Some(PathBuf::from("the-wait-path")),
                }),
                skip_document_code: Some(123),
            },
        };
        assert_eq!(
            serde_yaml::to_string(&config).expect("render document config to YAML"),
            FULL_DOCUMENT_CONFIG.to_string().trim_start(),
        )
    }

    const FULL_TESTCASE_CONFIG: &str = "
detached: true
environment:
  BAZ: zoing
  FOO: bar
keep_crlf: true
output_stream: stderr
skip_document_code: 123
timeout: 6m 4s
wait:
  timeout: 2m 1s
  path: the-wait-path
";

    #[test]
    fn test_parse_full_testcase_config() {
        let config: TestCaseConfig =
            serde_yaml::from_str(FULL_TESTCASE_CONFIG).expect("parse full testcase config");
        assert_eq!(
            config,
            TestCaseConfig {
                output_stream: Some(OutputStreamControl::Stderr),
                keep_crlf: Some(true),
                timeout: Some(Duration::from_secs(6 * 60 + 4)),
                environment: {
                    let mut m = BTreeMap::new();
                    m.insert("FOO".to_string(), "bar".to_string());
                    m.insert("BAZ".to_string(), "zoing".to_string());
                    m
                },
                detached: Some(true),
                wait: Some(TestCaseWait {
                    timeout: Duration::from_secs(2 * 60 + 1),
                    path: Some(PathBuf::from("the-wait-path")),
                }),
                skip_document_code: Some(123),
            }
        )
    }

    #[test]
    fn test_render_full_testcase_config() {
        let config = TestCaseConfig {
            output_stream: Some(OutputStreamControl::Stderr),
            keep_crlf: Some(true),
            timeout: Some(Duration::from_secs(6 * 60 + 4)),
            environment: {
                let mut m = BTreeMap::new();
                m.insert("FOO".to_string(), "bar".to_string());
                m.insert("BAZ".to_string(), "zoing".to_string());
                m
            },
            detached: Some(true),
            wait: Some(TestCaseWait {
                timeout: Duration::from_secs(2 * 60 + 1),
                path: Some(PathBuf::from("the-wait-path")),
            }),
            skip_document_code: Some(123),
        };
        assert_eq!(
            serde_yaml::to_string(&config).expect("render testcase config to YAML"),
            FULL_TESTCASE_CONFIG.to_string().trim_start(),
        )
    }

    #[test]
    fn test_parse_test_case_wait() {
        let tests = vec![
            (
                "wait: 3m 4s",
                Some(TestCaseWait {
                    timeout: Duration::from_secs(3 * 60 + 4),
                    path: None,
                }),
            ),
            (
                "wait:\n    timeout: 3m 5s\n    path: some/file/name",
                Some(TestCaseWait {
                    timeout: Duration::from_secs(3 * 60 + 5),
                    path: Some(PathBuf::from("some/file/name")),
                }),
            ),
        ];
        for (raw, expect) in tests {
            let config: TestCaseConfig =
                serde_yaml::from_str(raw).unwrap_or_else(|err| panic!("parse {raw:?}: {err}"));
            assert_eq!(config.wait, expect, "for input {raw:?}");
        }
    }
}
