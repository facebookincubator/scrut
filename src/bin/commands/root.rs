/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use scrut::config::DocumentConfig;
use scrut::config::OutputStreamControl;
use scrut::config::TestCaseConfig;
use scrut::escaping::Escaper;
use scrut::parsers::parser::ParserType;

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create(super::create::Args),
    Test(super::test::Args),
    Update(super::update::Args),
}

impl Commands {
    pub(crate) fn run(&self) -> anyhow::Result<()> {
        match &self {
            Commands::Create(cmd) => cmd.run(),
            Commands::Test(cmd) => cmd.run(),
            Commands::Update(cmd) => cmd.run(),
        }
    }
}

/// Supported scrut output format
#[derive(Debug, Clone, ValueEnum)]
pub enum ScrutRenderer {
    Auto,
    Pretty,
    Diff,
    Json,
    Yaml,
}

#[derive(Parser, Debug)]
pub(crate) struct GlobalParameters {
    /// Do things to be as compatible as possible with Cram:
    /// Inject CRAM* environment variables.
    /// Use glob matcher that supports escaped wildcards.
    /// Enable the --combine-output parameter.
    /// Enable the --keep-output-crlf parameter.
    #[clap(long, short = 'C', global = true)]
    pub(crate) cram_compat: bool,

    /// Per default only STDOUT will be considered. This flags combines STDOUT
    /// and STDERR into a single stream.
    #[clap(long, overrides_with = "no_combine_output", global = true)]
    pub(crate) combine_output: bool,
    #[clap(long, overrides_with = "combine_output", global = true)]
    pub(crate) no_combine_output: bool,

    /// Per default all CRLF line endings from outputs of shell expressions will
    /// be converted into LF line endings and need not be considered in output
    /// expectations. This flag surfaces CRLF line endings so that they can (and
    /// must be) addressed in output expectations (e.g. `output line\r (escaped)`)
    #[clap(long, overrides_with = "no_keep_output_crlf", global = true)]
    pub(crate) keep_output_crlf: bool,
    #[clap(long, overrides_with = "keep_output_crlf", global = true)]
    pub(crate) no_keep_output_crlf: bool,

    /// Optional output escaping mode. If not set then defaults to escaping
    /// all non-printable unicode characters for Scrut Markdown tests and
    /// all non-printable ASCII characters for Cram tests.
    #[clap(long, short = 'e', global = true)]
    pub(crate) escaping: Option<Escaper>,

    /// Shell to execute expressions in
    #[clap(long, short, global = true)]
    pub(crate) shell: Option<PathBuf>,

    /// Optional path to work directory in which the tests will be executed. Per
    /// default a temporary work directory for each test file will be created
    /// instead.
    #[clap(long, short, global = true)]
    pub(crate) work_directory: Option<PathBuf>,

    /// Whether not to clean up temporary directories after test execution
    #[clap(long, conflicts_with = "work_directory", global = true)]
    pub(crate) keep_temporary_directories: bool,

    /// Timeout in seconds for whole execution. Use 0 for unlimited. Defaults to 900, if not set.
    #[clap(long, global = true)]
    pub(crate) timeout_seconds: Option<u64>,
}

#[derive(Parser, Debug, Default)]
pub(crate) struct GlobalSharedParameters {
    #[clap(from_global)]
    pub(crate) cram_compat: bool,

    #[clap(from_global)]
    pub(crate) combine_output: bool,
    #[clap(from_global)]
    pub(crate) no_combine_output: bool,

    #[clap(from_global)]
    pub(crate) keep_output_crlf: bool,
    #[clap(from_global)]
    pub(crate) no_keep_output_crlf: bool,

    #[clap(from_global)]
    pub(crate) shell: Option<PathBuf>,

    #[clap(from_global)]
    pub(crate) work_directory: Option<PathBuf>,

    #[clap(from_global)]
    pub(crate) keep_temporary_directories: bool,

    #[clap(from_global)]
    pub(crate) escaping: Option<Escaper>,

    #[clap(from_global)]
    pub(crate) timeout_seconds: Option<u64>,
}

impl GlobalSharedParameters {
    /// Translates global shared parameters into (defaults for) per-document configuration
    pub(crate) fn to_document_config(&self) -> DocumentConfig {
        let mut config = DocumentConfig::empty();
        if let Some(ref value) = self.shell {
            config.shell = Some(value.clone())
        }
        if let Some(value) = self.timeout_seconds {
            config.total_timeout = Some(Duration::from_secs(value))
        }

        config
    }

    /// Translates global shared parameters into (defaults for) per-test configuration
    pub(crate) fn to_testcase_config(&self) -> TestCaseConfig {
        let mut config = TestCaseConfig::empty();

        // look at which output stream(s)
        // TODO: The new default here is supposed to be [`OutputStreamControl::Combined`].
        //       Make sure all current use `--no-combine-output` flag.
        if self.no_combine_output {
            config.output_stream = Some(OutputStreamControl::Stdout)
        } else if self.combine_output {
            config.output_stream = Some(OutputStreamControl::Combined)
        }

        // keep CRLF or replace to LF?
        if self.no_keep_output_crlf {
            config.keep_crlf = Some(false)
        } else if self.keep_output_crlf {
            config.keep_crlf = Some(true)
        }

        config
    }

    pub(crate) fn output_escaping(&self, parser: Option<ParserType>) -> Escaper {
        self.escaping
            .to_owned()
            .unwrap_or_else(|| match parser.unwrap_or(ParserType::Markdown) {
                ParserType::Markdown => Escaper::Unicode,
                ParserType::Cram => Escaper::Ascii,
            })
    }
}

#[cfg(test)]
mod tests {
    use scrut::config::DocumentConfig;
    use scrut::config::OutputStreamControl;
    use scrut::config::TestCaseConfig;

    use super::GlobalSharedParameters;

    #[test]
    fn test_to_document_config() {
        let tests = vec![
            (GlobalSharedParameters::default(), DocumentConfig::empty()),
            (
                GlobalSharedParameters {
                    shell: Some("other-shell".into()),
                    ..Default::default()
                },
                DocumentConfig {
                    shell: Some("other-shell".into()),
                    ..DocumentConfig::empty()
                },
            ),
        ];

        for (params, expected) in tests {
            assert_eq!(params.to_document_config(), expected);
        }
    }

    #[test]
    fn test_to_testcase_config() {
        let tests = vec![
            (GlobalSharedParameters::default(), TestCaseConfig::empty()),
            (
                GlobalSharedParameters {
                    no_combine_output: true,
                    ..Default::default()
                },
                TestCaseConfig {
                    output_stream: Some(OutputStreamControl::Stdout),
                    ..TestCaseConfig::empty()
                },
            ),
            (
                GlobalSharedParameters {
                    combine_output: true,
                    ..Default::default()
                },
                TestCaseConfig {
                    output_stream: Some(OutputStreamControl::Combined),
                    ..TestCaseConfig::empty()
                },
            ),
            (
                GlobalSharedParameters {
                    no_keep_output_crlf: true,
                    ..Default::default()
                },
                TestCaseConfig {
                    keep_crlf: Some(false),
                    ..TestCaseConfig::empty()
                },
            ),
            (
                GlobalSharedParameters {
                    keep_output_crlf: true,
                    ..Default::default()
                },
                TestCaseConfig {
                    keep_crlf: Some(true),
                    ..TestCaseConfig::empty()
                },
            ),
        ];

        for (idx, (params, expected)) in tests.into_iter().enumerate() {
            assert_eq!(
                params.to_testcase_config(),
                expected,
                "test case #{}",
                idx + 1
            );
        }
    }
}
