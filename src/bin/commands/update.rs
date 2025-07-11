/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BTreeMap;
use std::fs;
use std::io::IsTerminal;
use std::io::stdout;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use dialoguer::console;
use dialoguer::console::style;
use scrut::config::DEFAULT_SKIP_DOCUMENT_CODE;
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
use scrut::renderers::pretty::DEFAULT_MULTILINE_MATCHED_LINES;
use scrut::renderers::pretty::DEFAULT_SURROUNDING_LINES;
use scrut::renderers::pretty::PrettyColorRenderer;
use scrut::renderers::pretty::PrettyMonochromeRenderer;
use scrut::renderers::renderer::Renderer;
use scrut::testcase::TestCase;

use super::root::GlobalSharedParameters;
use crate::utils::FileParser;
use crate::utils::ParsedTestFile;
use crate::utils::ProgressWriter;
use crate::utils::TestEnvironment;
use crate::utils::canonical_shell;
use crate::utils::confirm;
use crate::utils::debug_testcases;
use crate::utils::get_log_level;
use crate::utils::make_executor;

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

    /// Specifies the number of lines to display when tests with multiline expectations
    /// fail when using the pretty renderer. If the number of matched lines exceeds
    /// this value, the extra lines will be truncated in the output.
    #[clap(long, default_value_t = DEFAULT_MULTILINE_MATCHED_LINES)]
    max_multiline_matched_lines: usize,

    /// Optional explicit format, in case the intention is to convert a test.
    /// If set then --output-suffix is ignored (new format file extension
    /// is used instead).
    /// Has no effect if the same format is chosen that the input test file
    /// already has.
    #[clap(long, short, value_enum)]
    convert: Option<ParserType>,

    /// Increase output verbosity, print out information that is not warning or errors
    #[clap(long)]
    verbose: bool,

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

        if tests.is_empty() {
            println!("👋 No test documents found in {:?}. Stopping.", &self.paths);
            return Ok(());
        }

        let document_config = self.to_document_config();
        let testcase_config = self.to_testcase_config();

        let pw = ProgressWriter::try_new(
            tests.len() as u64,
            get_log_level() <= tracing::Level::WARN,
            self.global.no_color || !console::colors_enabled(),
        )?;

        // iterate each test file
        pw.println(format!("🔎 Found {} test document(s)", tests.len()));
        let (mut count_updated, mut count_unchanged, mut count_skipped) = (0, 0, 0);
        for mut test in tests {
            pw.inc(1);
            pw.set_message(format!(
                "👀 {}",
                style(test.path.to_string_lossy()).yellow()
            ));

            let config = test.config.with_overrides_from(&document_config);
            let shell_path = canonical_shell(config.shell.as_ref().map(|p| p as &Path))?;

            let mut test_environment = TestEnvironment::new(
                &shell_path,
                self.global.work_directory.as_deref(),
                self.global.keep_temporary_directories,
            )?;

            // must have test-cases to continue
            if test.testcases.is_empty() {
                count_skipped += 1;
                pw.println(format!(
                    "⏩ {}: skipped, because no testcases were found in the document",
                    style(test.path.to_string_lossy()).blue()
                ));
                continue;
            }

            // TODO(config): Add support for updating prepended and appended files (or reason why not)
            if !config.prepend.is_empty() || !config.prepend.is_empty() {
                count_skipped += 1;
                pw.println(format!(
                    "⏩ {}: skipped, because 'prepend' or 'append' are currently not supported in update",
                    style(test.path.to_string_lossy()).blue()
                ));
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
                        .with_environment(&env_vars);
                    testcase as &TestCase
                })
                .collect::<Vec<_>>();

            // get the appropriate or requested executor
            let executor = make_executor(&test_environment.shell, cram_compat)?;

            // execute the tests to use the updated result to update the test file
            let execution_result = executor.execute_all(
                &testcases,
                &ContextBuilder::default()
                    .work_directory(PathBuf::from(&test_work_directory))
                    .temp_directory(test_environment.tmp_directory.as_path_buf())
                    .file(test.path.clone())
                    .config(test.config.with_overrides_from(&document_config))
                    .build()
                    .context("failed to build execution context")?,
            );
            match execution_result {
                // test execution failed ..
                Err(err) => match err {
                    // .. intentionally with skip, so skip
                    ExecutionError::Skipped(idx) => {
                        count_skipped += 1;
                        pw.println(format!(
                            "⏩ {}: skipped, because testcase #{} ended in exit code {}",
                            style(test.path.to_string_lossy()).blue(),
                            idx + 1,
                            testcases.get(idx).map_or(DEFAULT_SKIP_DOCUMENT_CODE, |t| t
                                .config
                                .get_skip_document_code())
                        ));
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
                        let mut testcase = testcase.to_owned();
                        testcase.config = testcase.config.without_environment(&env_vars);
                        outcomes.push(Outcome {
                            testcase,
                            location: Some(test.path.to_string_lossy().to_string()),
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
                    let is_conversion = self.convert.is_some_and(|c| c != test.parser_type);
                    let (updated, output_type) = if is_conversion {
                        self.convert_test(&test, outcomes)
                    } else {
                        self.update_test(&test, outcomes)
                    }?;

                    // .. without changes -> next plz
                    if updated == test.content {
                        count_unchanged += 1;
                        if self.verbose {
                            pw.println(format!(
                                "👍 {}: keep as-is, no changes in document content",
                                style(test.path.to_string_lossy()).blue()
                            ));
                        }
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
                    if !self.assume_yes && Path::new(&output_path).exists() {
                        let confirmed = pw.suspend(|| {
                            confirm(
                                &format!(
                                    "Overwrite existing document {}?",
                                    style(output_path.to_string_lossy()).blue()
                                ),
                                false,
                                self.global.no_color,
                            )
                        })?;

                        if !confirmed {
                            //eprintln!("  Skipping!");
                            count_skipped += 1;

                            pw.println(format!(
                                "👎 {}: keep as-is, chosen not to overwrite document",
                                style(test.path.to_string_lossy()).red()
                            ));
                            continue;
                        }
                    }

                    count_updated += 1;
                    fs::write(&output_path, &updated).with_context(|| {
                        format!("overwrite existing document in {:?}", test.path)
                    })?;
                    if output_path == test.path {
                        pw.println(format!(
                            "✍️ {}: overwritten document with updated contents",
                            style(test.path.to_string_lossy()).green()
                        ));
                    } else {
                        pw.println(format!(
                            "🌟 {}: updated document contents written to {}",
                            style(test.path.to_string_lossy()).green(),
                            style(output_path.to_string_lossy()).blue()
                        ));
                    }
                }
            }
        }
        pw.println("");
        pw.finish_and_clear();

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
            .with_context(|| {
                format!(
                    "generating update for testcases in document {:?}",
                    test.path
                )
            })?;
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

        let generated = generator.generate_testcases(outcomes).with_context(|| {
            format!(
                "generating conversion for testcases in document {:?}",
                test.path
            )
        })?;

        Ok((generated, parser_type))
    }

    fn print_changes(&self, outcomes: &[&Outcome]) {
        let color_renderer = PrettyColorRenderer {
            max_surrounding_lines: DEFAULT_SURROUNDING_LINES,
            absolute_line_numbers: self.absolute_line_numbers,
            summarize: false,
            max_multiline_matched_lines: self.max_multiline_matched_lines,
        };
        let diff: Box<dyn Renderer> = if self.global.no_color {
            Box::new(PrettyMonochromeRenderer::new(color_renderer))
        } else {
            Box::new(color_renderer)
        };
        eprint!("{}", diff.render(outcomes).expect("outcomes rendered"));
    }

    fn render_summary(&self, updated: usize, skipped: usize, unchanged: usize) -> String {
        let summary = style("Result").underlined();
        let total = updated + skipped + unchanged;
        let files = style(format!("{} document(s)", total)).bold();
        let mut updated_fmt = style(format!("{} updated", updated)).green();
        if updated > 0 {
            updated_fmt = updated_fmt.bold();
        }
        let mut skipped_fmt = style(format!("{} skipped", skipped)).yellow();
        if skipped > 0 {
            skipped_fmt = skipped_fmt.bold();
        }
        let mut unchanged_fmt = style(format!("{} unchanged", unchanged)).magenta();
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
        if self.global.no_color || !stdout().is_terminal() {
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
