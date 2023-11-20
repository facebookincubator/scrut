use std::collections::BTreeMap;
use std::fs;
use std::io::stdout;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;

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
use scrut::testcase::TestCase;
use tracing::debug;
use tracing::info;
use tracing::warn;

use super::root::GlobalSharedParameters;
use crate::utils::canonical_shell;
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

        let document_config = self.to_document_config();
        let testcase_config = self.to_testcase_config();

        // iterate each test file
        debug!("updating {} test files", tests.len());
        let (mut count_updated, mut count_unchanged, mut count_skipped) = (0, 0, 0);
        for mut test in tests {
            debug!("updating test file {:?}", test.path);

            let config = test.config.with_overrides_from(&document_config);
            let shell_path = canonical_shell(config.shell.as_ref().map(|p| p as &Path))?;

            let mut test_environment =
                TestEnvironment::new(&shell_path, self.global.work_directory.as_deref())?;

            // must have test-cases to continue
            if test.testcases.is_empty() {
                warn!(
                    "Ignoring file {:?} that does not contain any testcases",
                    &test.path
                );
                count_skipped += 1;
                continue;
            }

            // TODO(config): Add support for updating prepended and appended files (or reason why not)
            if !config.prepend.is_empty() || !config.prepend.is_empty() {
                warn!(
                    "Skipping file {:?} that contains 'prepend' or 'append' configuration, which is currently not supported in update command",
                    &test.path
                );
                count_skipped += 1;
                continue;
            }

            // setup test file environment ..
            let cram_compat = test.parser_type == ParserType::Cram;
            let (test_work_directory, env_vars) =
                test_environment.init_test_file(&test.path, cram_compat)?;

            // extract testcases and update with config from parameters
            let env_vars =
                BTreeMap::from_iter(env_vars.iter().map(|(k, v)| (k as &str, v as &str)));
            let testcases = test
                .testcases
                .iter_mut()
                .map(|testcase| {
                    testcase.config = testcase
                        .config
                        .with_overrides_from(&testcase_config)
                        .merge_environment(&env_vars);
                    testcase as &TestCase
                })
                .collect::<Vec<_>>();

            // get the appropriate or requested executor
            let executor = make_executor(&test_environment.shell, cram_compat)?;

            // execute the tests to use the updated result to update the test file
            let execution_result = executor.execute_all(
                &testcases,
                &ContextBuilder::default()
                    .work_directory(Some(PathBuf::from(&test_work_directory)))
                    .temp_directory(Some(test_environment.tmp_directory.as_path_buf()))
                    .config(test.config.with_overrides_from(&document_config))
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

                    // TODO: continue on ExecutionError::Timeout, but warn! or error!

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
        self.global.to_document_config()
    }

    fn to_testcase_config(&self) -> TestCaseConfig {
        self.global.to_testcase_config()
    }
}
