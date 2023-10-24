use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use scrut::config::DocumentConfig;
use scrut::config::OutputStreamControl;
use scrut::config::TestCaseConfig;
use scrut::escaping::Escaper;
use scrut::executors::DEFAULT_SHELL;
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
    #[clap(long, short, default_value = (&*DEFAULT_SHELL).to_string_lossy().to_string(), global = true)]
    pub(crate) shell: PathBuf,

    /// Optional path to work directory in which the tests will be executed. Per
    /// default a temporary work directory for each test file will be created
    /// instead.
    #[clap(long, short, global = true)]
    pub(crate) work_directory: Option<PathBuf>,
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
    pub(crate) shell: PathBuf,

    #[clap(from_global)]
    pub(crate) work_directory: Option<PathBuf>,

    #[clap(from_global)]
    pub(crate) escaping: Option<Escaper>,
}

impl GlobalSharedParameters {
    pub(crate) fn is_combine_output(&self, parser: Option<ParserType>) -> bool {
        self.combine_output || self.cram_compat || parser == Some(ParserType::Cram)
    }

    /// Translates global shared parameters into (defaults for) per-document configuration
    pub(crate) fn to_document_config(&self) -> DocumentConfig {
        let mut config = DocumentConfig::empty();
        if !self.shell.as_os_str().is_empty() {
            config.shell = Some(self.shell.clone())
        }

        config
    }

    pub(crate) fn is_keep_output_crlf(&self, parser: Option<ParserType>) -> bool {
        self.keep_output_crlf || self.cram_compat || parser == Some(ParserType::Cram)
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
    fn test_as_document_config() {
        let tests = vec![
            (GlobalSharedParameters::default(), DocumentConfig::empty()),
            (
                GlobalSharedParameters {
                    shell: "other-shell".into(),
                    ..Default::default()
                },
                DocumentConfig {
                    shell: Some("other-shell".into()),
                    ..Default::default()
                },
            ),
        ];

        for (params, expected) in tests {
            assert_eq!(params.to_document_config(), expected);
        }
    }

    #[test]
    fn test_as_testcase_config() {
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
