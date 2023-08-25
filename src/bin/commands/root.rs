use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
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
    #[clap(long, global = true)]
    pub(crate) combine_output: bool,

    /// Per default all CRLF line endings from outputs of shell expressions will
    /// be converted into LF line endings and need not be considered in output
    /// expectations. This flag surfaces CRLF line endings so that they can (and
    /// must be) addressed in output expectations (e.g. `output line\r (escaped)`)
    #[clap(long, global = true)]
    pub(crate) keep_output_crlf: bool,

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
    pub(crate) keep_output_crlf: bool,

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

    pub(crate) fn is_keep_output_crlf(&self, parser: Option<ParserType>) -> bool {
        self.keep_output_crlf || self.cram_compat || parser == Some(ParserType::Cram)
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
    use scrut::parsers::parser::ParserType;

    use super::GlobalSharedParameters;

    #[test]
    fn test_combine_output() {
        let tests = &[
            (
                false,
                "all default",
                GlobalSharedParameters {
                    ..Default::default()
                },
                None,
            ),
            (
                false,
                "all default, markdown parser",
                GlobalSharedParameters {
                    ..Default::default()
                },
                Some(ParserType::Markdown),
            ),
            (
                true,
                "all default, cram parser",
                GlobalSharedParameters {
                    ..Default::default()
                },
                Some(ParserType::Cram),
            ),
            (
                true,
                "combine output enabled",
                GlobalSharedParameters {
                    combine_output: true,
                    ..Default::default()
                },
                None,
            ),
            (
                true,
                "cram compat enabled",
                GlobalSharedParameters {
                    cram_compat: true,
                    ..Default::default()
                },
                None,
            ),
            (
                true,
                "both enabled",
                GlobalSharedParameters {
                    combine_output: true,
                    cram_compat: true,
                    ..Default::default()
                },
                None,
            ),
        ];

        for (expect, description, params, parser_type) in tests {
            assert!(
                *expect == params.is_combine_output(*parser_type),
                "{}",
                *description
            )
        }
    }

    #[test]
    fn test_keep_output_crlf() {
        let tests = &[
            (
                false,
                "all default",
                GlobalSharedParameters {
                    ..Default::default()
                },
                None,
            ),
            (
                false,
                "all default, markdown parser",
                GlobalSharedParameters {
                    ..Default::default()
                },
                Some(ParserType::Markdown),
            ),
            (
                true,
                "all default, cram parser",
                GlobalSharedParameters {
                    ..Default::default()
                },
                Some(ParserType::Cram),
            ),
            (
                true,
                "keep output crlf enabled",
                GlobalSharedParameters {
                    keep_output_crlf: true,
                    ..Default::default()
                },
                None,
            ),
            (
                true,
                "cram compat enabled",
                GlobalSharedParameters {
                    cram_compat: true,
                    ..Default::default()
                },
                None,
            ),
            (
                true,
                "both enabled",
                GlobalSharedParameters {
                    keep_output_crlf: true,
                    cram_compat: true,
                    ..Default::default()
                },
                None,
            ),
        ];

        for (expect, description, params, parser_type) in tests {
            assert!(
                *expect == params.is_keep_output_crlf(*parser_type),
                "{}",
                *description
            )
        }
    }
}
