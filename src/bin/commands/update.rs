use std::fs;
use std::io::stdout;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use scrut::config::DocumentConfig;
use scrut::config::TestCaseConfig;
use scrut::escaping::strip_colors;
use scrut::executors::context::ContextBuilder;
use scrut::executors::error::ExecutionError;
use scrut::executors::execution::Execution;
use scrut::generators::cram::CramTestCaseGenerator;
use scrut::generators::cram::CramUpdateGenerator;
use scrut::generators::generator::TestCaseGenerator;
use scrut::generators::generator::UpdateGenerator;
use scrut::generators::markdown::MarkdownTestCaseGenerator;
use scrut::generators::markdown::MarkdownUpdateGenerator;
use scrut::outcome::Outcome;
use scrut::parsers::markdown::DEFAULT_MARKDOWN_LANGUAGES;
use scrut::parsers::parser::ParserType;
use scrut::renderers::pretty::PrettyColorRenderer;
use scrut::renderers::pretty::PrettyMonochromeRenderer;
use scrut::renderers::pretty::DEFAULT_SURROUNDING_LINES;
use scrut::renderers::renderer::Renderer;
use tracing::debug;
use tracing::info;
use tracing::warn;

use super::root::GlobalSharedParameters;
use crate::utils::confirm;
use crate::utils::debug_testcases;
use crate::utils::make_executor;
use crate::utils::FileParser;
use crate::utils::ParsedTestFile;
use crate::utils::TestEnvironment;

/// Re-run all testcases in given file(s) and update the output expectations
#[derive(Debug, Parser)]
pub struct Args {
    /// Path to test files or directories
    #[clap(required = true)]
    paths: Vec<PathBuf>,

    /// Whether to print out debug output - use only
    #[clap(long)]
    debug: bool,

    /// For markdown format: Language annotations that are considered test cases
    #[clap(long, hide = true, default_values = DEFAULT_MARKDOWN_LANGUAGES, num_args=1..)]
    markdown_languages: Vec<String>,

    /// Per default colo(u)r output is enabled on TTYs when the `diff` renderer
    /// is used. This flag disables colo(u)r output in that case
    #[clap(long, alias = "no-colour")]
    no_color: bool,

    /// What suffix to add to thew newly created file (will overwrite already
    /// existing files!)
    #[clap(long, short, default_value = ".new")]
    output_suffix: String,

    /// Danger! Whether to assume Yes for the question to overwrite files when
    /// with updated contents. In conjunction with the `--replace` flag this
    /// means the original file will be overwritten.
    #[clap(long, short = 'y', aliases = &["overwrite-all"])]
    assume_yes: bool,

    /// Glob match that identifies cram files
    #[clap(long, default_value = "*.{t,cram}")]
    match_cram: String,

    /// Glob match that identifies markdown files
    #[clap(long, default_value = "*.{md,markdown}")]
    match_markdown: String,

    /// Whether to replace the contents of the files (see --output-suffix)
    #[clap(long, short)]
    replace: bool,

    /// For sequential: Timeout in seconds for whole execution. Use 0 for unlimited
    #[clap(long, short = 'S', default_value = "900")]
    timeout_seconds: usize,

    /// Per default, renderers that provide line numbers use relative numbers within
    /// the test case / the output of the execution. Setting this flag changes that
    /// to use absolute line numbers from within the test file.
    #[clap(long)]
    absolute_line_numbers: bool,

    /// Optional explicit format, in case the intention is to convert a test.
    /// If set then --output-suffix is ignored (new format file extension
    /// is used instead).
    /// Has no effect if the same format is chosen that the input test file
    /// already has.
    #[clap(long, short, value_enum)]
    convert: Option<ParserType>,

    #[clap(flatten)]
    global: GlobalSharedParameters,
}

impl Args {
    pub(crate) fn run(&self) -> Result<()> {
        // init parser and determine suffices to look for
        let markdown_languages = &self
            .markdown_languages
            .iter()
            .map(|s| &**s)
            .collect::<Vec<_>>();
        let parser = FileParser::new(&self.match_markdown, &self.match_cram, markdown_languages)
            .context("create file parser")?;

        let tests = parser.find_and_parse(
            "test",
            &self.paths.iter().map(|p| p as &Path).collect::<Vec<_>>(),
            self.global.cram_compat,
        )?;

        let mut test_environment =
            TestEnvironment::new(&self.global.shell, self.global.work_directory.as_deref())?;

        // iterate each test file
        debug!("updating {} test files", tests.len());
        let (mut count_updated, mut count_unchanged, mut count_skipped) = (0, 0, 0);
        for test in tests {
            debug!("updating test file {:?}", test.path);

            // setup test file environment ..
            let cram_compat = test.parser_type == ParserType::Cram;
            let (test_work_directory, environment) =
                test_environment.init_test_file(&test.path, cram_compat)?;

            // must have test-cases to continue
            if test.testcases.is_empty() {
                warn!(
                    "Ignoring file {:?} that does not contain any testcases",
                    &test.path
                );
                count_skipped += 1;
                continue;
            }

            // run the test cases, get the output
            let executions = test
                .testcases
                .iter()
                .map(|a| {
                    Execution::new(&a.shell_expression).environment(
                        &environment
                            .iter()
                            .map(|(k, v)| (k as &str, v as &str))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>();

            let (timeout, executor) =
                make_executor(&self.global.shell, self.timeout_seconds, cram_compat)?;

            let execution_result = executor.execute_all(
                &executions.iter().collect::<Vec<_>>(),
                &ContextBuilder::default()
                    .combine_output(self.global.is_combine_output(Some(test.parser_type)))
                    .crlf_support(self.global.is_keep_output_crlf(Some(test.parser_type)))
                    .work_directory(Some(PathBuf::from(&test_work_directory)))
                    .timeout(timeout)
                    .build()
                    .context("failed to build execution context")?,
            );
            match execution_result {
                // test execution failed ..
                Err(err) => match err {
                    // .. intentionally with skip, so skip
                    ExecutionError::Skipped(_) => {
                        count_skipped += 1;
                        info!("Skipping test file {:?}", &test.path);
                        continue;
                    }
                    // .. unintentionally with an error -> give up
                    _ => bail!("failing in {:?}: {}", test.path, err),
                },

                // test execution succeeded
                Ok(outputs) => {
                    let mut outcomes = vec![];

                    // take test execution output, run validation and store all outcomes ...
                    for (testcase, output) in test.testcases.iter().zip(outputs.iter()) {
                        let result = testcase.validate(output);
                        outcomes.push(Outcome {
                            location: Some(test.path.to_string_lossy().to_string()),
                            testcase: testcase.to_owned(),
                            output: output.to_owned(),
                            escaping: self.global.output_escaping(Some(test.parser_type)),
                            format: test.parser_type,
                            result,
                        });
                    }

                    if self.debug {
                        debug_testcases(&test.testcases, &test.path, &outputs);
                    }

                    // .. and create an updated content (either from actual update or conversion)
                    let outcomes = &outcomes.iter().collect::<Vec<_>>();
                    let is_conversion = self.convert.map_or(false, |c| c != test.parser_type);
                    let (updated, output_type) = if is_conversion {
                        self.convert_test(&test, outcomes)
                    } else {
                        self.update_test(&test, outcomes)
                    }?;

                    // .. without changes -> next plz
                    if updated == test.content {
                        count_unchanged += 1;
                        info!("Keep unchanged test file {:?}", test.path);
                        continue;
                    }
                    self.print_changes(outcomes);

                    // determine new location
                    let output_path = if is_conversion {
                        let stripped_path = test
                            .path
                            .file_stem()
                            .map_or(&test.path as &Path, Path::new)
                            .to_path_buf();
                        stripped_path.with_extension(output_type.file_extension())
                    } else if self.replace {
                        test.path.clone()
                    } else {
                        let mut extension = vec![self.output_suffix.clone()];
                        if let Some(ext) = test.path.extension() {
                            extension.push(ext.to_string_lossy().to_string())
                        }
                        extension.reverse();
                        test.path.clone().with_extension(extension.join(""))
                    };

                    // always ask, in case the file exists
                    if !self.assume_yes
                        && Path::new(&output_path).exists()
                        && !confirm(&format!("> Overwrite existing file {:?}?", &output_path))?
                    {
                        eprintln!("  Skipping!");
                        count_skipped += 1;
                        continue;
                    }

                    count_updated += 1;
                    info!("Writing updated test file to {:?}", output_path);
                    fs::write(&output_path, &updated)
                        .with_context(|| format!("overwrite existing file in {:?}", test.path))?;
                }
            }
        }

        self.print_summary(count_updated, count_skipped, count_unchanged)?;

        Ok(())
    }

    fn update_test(
        &self,
        test: &ParsedTestFile,
        outcomes: &[&Outcome],
    ) -> Result<(String, ParserType)> {
        let generator: Box<dyn UpdateGenerator> = match test.parser_type {
            ParserType::Markdown => Box::new(MarkdownUpdateGenerator::new(
                &self
                    .markdown_languages
                    .iter()
                    .map(|s| s as &str)
                    .collect::<Vec<_>>(),
            )),
            ParserType::Cram => Box::<CramUpdateGenerator>::default(),
        };

        let generated = generator
            .generate_update(&test.content, outcomes)
            .with_context(|| format!("generating update for tests in file {:?}", test.path))?;
        Ok((generated, test.parser_type))
    }

    fn convert_test(
        &self,
        test: &ParsedTestFile,
        outcomes: &[&Outcome],
    ) -> Result<(String, ParserType)> {
        let (generator, parser_type): (Box<dyn TestCaseGenerator>, ParserType) =
            match test.parser_type {
                ParserType::Markdown => (Box::<CramTestCaseGenerator>::default(), ParserType::Cram),
                ParserType::Cram => (
                    Box::new(MarkdownTestCaseGenerator::new(&self.markdown_languages[0])),
                    ParserType::Markdown,
                ),
            };

        let generated = generator
            .generate_testcases(outcomes)
            .with_context(|| format!("generating conversion for tests in file {:?}", test.path))?;

        Ok((generated, parser_type))
    }

    fn print_changes(&self, outcomes: &[&Outcome]) {
        let diff: Box<dyn Renderer> = if self.no_color {
            Box::new(PrettyMonochromeRenderer::new(
                DEFAULT_SURROUNDING_LINES,
                self.absolute_line_numbers,
            ))
        } else {
            Box::new(PrettyColorRenderer::new(
                DEFAULT_SURROUNDING_LINES,
                self.absolute_line_numbers,
            ))
        };
        eprint!("{}", diff.render(outcomes).expect("outcomes rendered"));
    }

    fn render_summary(&self, updated: usize, skipped: usize, unchanged: usize) -> String {
        let summary = "Summary".underline();
        let total = updated + skipped + unchanged;
        let files = format!("{} file(s)", total).bold();
        let mut updated_fmt = format!("{} updated", updated).green();
        if updated > 0 {
            updated_fmt = updated_fmt.bold();
        }
        let mut skipped_fmt = format!("{} skipped", skipped).yellow();
        if skipped > 0 {
            skipped_fmt = skipped_fmt.bold();
        }
        let mut unchanged_fmt = format!("{} unchanged", unchanged).magenta();
        if unchanged > 0 {
            unchanged_fmt = unchanged_fmt.bold();
        }
        format!(
            "{}: {} of which {}, {} and {}",
            summary, files, updated_fmt, skipped_fmt, unchanged_fmt,
        )
    }

    fn print_summary(&self, updated: usize, skipped: usize, unchanged: usize) -> Result<()> {
        let mut summary = self.render_summary(updated, skipped, unchanged);
        if self.no_color || !stdout().is_terminal() {
            summary = strip_colors(&summary)?;
        }
        println!("{}", summary);
        Ok(())
    }

    fn to_document_config(&self) -> DocumentConfig {
        let mut config = DocumentConfig::empty();
        if self.timeout_seconds > 0 {
            config.total_timeout = Some(Duration::from_secs(self.timeout_seconds as u64))
        }

        config.with_defaults_from(&self.global.to_document_config())
    }

    fn to_testcase_config(&self) -> TestCaseConfig {
        self.global.to_testcase_config()
    }
}
