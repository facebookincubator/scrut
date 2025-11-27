/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use clap::Parser as ClapParser;
use dialoguer::console::style;
use humantime::format_duration;
use scrut::config::DEFAULT_SKIP_DOCUMENT_CODE;
use scrut::config::DocumentConfig;
use scrut::config::TestCaseConfig;
use scrut::executors::context::ContextBuilder;
use scrut::executors::error::ExecutionError;
use scrut::executors::error::ExecutionTimeout;
use scrut::outcome::Outcome;
use scrut::output::ExitStatus;
use scrut::parsers::markdown::DEFAULT_MARKDOWN_LANGUAGES;
use scrut::parsers::parser::ParserType;
use scrut::renderers::diff::DiffRenderer;
use scrut::renderers::pretty::DEFAULT_MULTILINE_MATCHED_LINES;
use scrut::renderers::pretty::DEFAULT_SURROUNDING_LINES;
use scrut::renderers::pretty::PrettyColorRenderer;
use scrut::renderers::pretty::PrettyMonochromeRenderer;
use scrut::renderers::renderer::Renderer;
use scrut::renderers::structured::JsonRenderer;
use scrut::renderers::structured::YamlRenderer;
use scrut::testcase::TestCase;
use scrut::testcase::TestCaseError;
use tracing::debug;
use tracing::debug_span;
use tracing::info;
use tracing::trace;

use super::root::GlobalSharedParameters;
use super::root::ScrutRenderer;
use crate::utils::FileParser;
use crate::utils::ProgressWriter;
use crate::utils::TestEnvironment;
use crate::utils::canonical_shell;
use crate::utils::debug_testcases;
use crate::utils::get_log_level;
use crate::utils::kill_detached_process;
use crate::utils::make_executor;

#[derive(Debug, thiserror::Error)]
#[error("validation failed")]
pub struct ValidationFailedError;

/// Run tests from files or directories
#[derive(Debug, ClapParser)]
pub struct Args {
    /// Path to test files or directories
    test_file_paths: Vec<PathBuf>,

    /// Optional list of paths to test files which are prepended to each test
    /// file in execution. Think: shared test bootstrap.
    /// This is NOT meant to be used from the command line, aside from
    /// edge-cases, for it breaks the containment of test files. Use the
    /// configuration file and persist it together with your tests instead.
    /// UNSTABLE: this parameter may change or be removed.
    #[clap(long, short = 'P', num_args=0..)]
    prepend_test_file_paths: Vec<PathBuf>,

    /// Optional list of paths to test files which are appended to each test
    /// file in execution. Think: shared test teardown.
    /// This is NOT meant to be used from the command line, aside from
    /// edge-cases, for it breaks the containment of test files. Use the
    /// configuration file and persist it together with your tests instead.
    /// UNSTABLE: this parameter may change or be removed.
    #[clap(long, short = 'A', num_args=0..)]
    append_test_file_paths: Vec<PathBuf>,

    /// Whether to print out debug output - use only
    #[clap(long)]
    debug: bool,

    /// For markdown format: Language annotations that are considered test cases
    #[clap(long, hide = true, default_values = DEFAULT_MARKDOWN_LANGUAGES, num_args = 1..)]
    markdown_languages: Vec<String>,

    /// Glob match that identifies cram files
    #[clap(long, default_value = "*.{t,cram}")]
    match_cram: String,

    /// Glob match that identifies markdown files
    #[clap(long, default_value = "*.{md,markdown,scrut}")]
    match_markdown: String,

    /// Which renderer to use for generating the result, with `diff` being the
    /// best choice for human consumption and `json` or `yaml` for further
    /// machine processing.
    #[clap(long, short, default_value = "auto", value_enum)]
    renderer: ScrutRenderer,

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
            &self
                .test_file_paths
                .iter()
                .map(|p| p as &Path)
                .collect::<Vec<_>>(),
            self.global.cram_compat,
        )?;

        // initiate outputs
        let mut outcomes = vec![];
        let (mut count_success, mut count_skipped, mut count_failed, mut count_detached) =
            (0, 0, 0, 0);

        // load configuration from command line
        let document_config = self.to_document_config();
        let testcase_config = self.to_testcase_config();
        let current_directory = std::env::current_dir().context("get current directory")?;

        let pw = ProgressWriter::try_new(
            tests.len() as u64,
            get_log_level() <= tracing::Level::WARN,
            self.global.no_color || !console::colors_enabled(),
        )?;
        pw.println(format!(
            "🔎 Found {} test document(s)",
            style(tests.len()).bold()
        ));

        for mut test in tests {
            pw.inc(1);
            pw.set_message(format!(
                "👀 {}",
                style(test.path.to_string_lossy()).yellow()
            ));

            // prefix append and prepend in document config with directory where test is
            let test_directory = &test.path.parent().unwrap_or(&current_directory);
            test.config.append = prefix_with_directory(test_directory, &test.config.append);
            test.config.prepend = prefix_with_directory(test_directory, &test.config.prepend);

            // compile configuration from test file and parameters
            let config: DocumentConfig = test.config.with_overrides_from(&document_config);

            // initialize environment in which test will run
            let shell_path = canonical_shell(config.shell.as_ref().map(|p| p as &Path))?;
            let mut test_environment = TestEnvironment::new(
                &shell_path,
                self.global.work_directory.as_deref(),
                self.global.keep_temporary_directories,
            )?;

            let span = debug_span!("test", path = %&test.path.display(), env = ?&test_environment);
            let _s = span.enter();

            // extract test cases from content ..
            debug!(
                format = %&test.parser_type,
                num_cases = &test.testcases.len(),
                config = %&config,
                "running tests",
            );

            // compile prepended and appended tests, based on both command line
            // parameters and the inline per-document configuration
            let prepend_tests = if !config.prepend.is_empty() {
                parser.find_and_parse(
                    "prepend test",
                    &config
                        .prepend
                        .iter()
                        .map(|p| p as &Path)
                        .collect::<Vec<_>>(),
                    self.global.cram_compat,
                )?
            } else {
                vec![]
            };
            let append_tests = if !config.append.is_empty() {
                parser.find_and_parse(
                    "append test",
                    &config.append.iter().map(|p| p as &Path).collect::<Vec<_>>(),
                    self.global.cram_compat,
                )?
            } else {
                vec![]
            };

            // gather executions from prepended, test file and appended
            let mut testcases = prepend_tests
                .iter()
                .flat_map(|parsed| parsed.testcases.clone())
                .collect::<Vec<_>>();
            testcases.extend(test.testcases.clone());
            testcases.extend(append_tests.iter().flat_map(|test| test.testcases.clone()));

            // setup testing environment
            let cram_compat = test.parser_type == ParserType::Cram || self.global.cram_compat;
            let (test_work_directory, env_vars) =
                test_environment.init_test_file(&test.path, cram_compat)?;

            // update testcase configuration from command line parameters
            let env_vars =
                BTreeMap::from_iter(env_vars.iter().map(|(k, v)| (k as &str, v as &str)));
            let testcases = testcases
                .iter_mut()
                .map(|testcase| {
                    testcase.config = testcase
                        .config
                        .with_overrides_from(&testcase_config)
                        .with_environment(&env_vars);
                    trace!(testcase = %&testcase, "running test case");
                    testcase as &TestCase
                })
                .collect::<Vec<_>>();

            // get the appropriate or requested executor
            let executor = make_executor(&test_environment.shell, cram_compat)?;

            // determine output escaping
            let escaping = self.global.output_escaping(Some(test.parser_type));

            // run all testcases from the file and gather output ..
            let outputs = executor.execute_all(
                testcases.as_slice(),
                &ContextBuilder::default()
                    .work_directory(PathBuf::from(&test_work_directory))
                    .temp_directory(test_environment.tmp_directory.as_path_buf())
                    .file(test.path.clone())
                    .config(config.clone())
                    .build()
                    .context("failed to build execution context")?,
            );
            match outputs {
                // test execution failed ...
                Err(err) => match err {
                    // ... because test was skipped
                    ExecutionError::Skipped(idx) => {
                        count_skipped += 1;
                        outcomes.extend(testcases.iter().map(|testcase| Outcome {
                            location: Some(test.path.display().to_string()),
                            testcase: (*testcase).clone(),
                            output: ("", "", None).into(),
                            escaping: escaping.clone(),
                            format: test.parser_type,
                            result: Err(TestCaseError::Skipped),
                        }));
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

                    // ... because test timed out
                    ExecutionError::Timeout(timeout, outputs) => {
                        handle_early_termination(
                            &outputs,
                            &testcases,
                            &mut outcomes,
                            test.path.display().to_string(),
                            escaping.clone(),
                            test.parser_type,
                            &mut count_success,
                            &mut count_failed,
                            &mut count_skipped,
                            |output, testcase| {
                                if matches!(output.exit_code, ExitStatus::Timeout(_)) {
                                    Err(TestCaseError::Timeout)
                                } else {
                                    testcase.validate(output)
                                }
                            },
                        );

                        let (location, timeout) = match timeout {
                            ExecutionTimeout::Index(idx) => (
                                format!("per-testcase timeout in testcase #{}", idx + 1),
                                testcases[idx].config.timeout,
                            ),
                            ExecutionTimeout::Total => {
                                ("per-document timeout".to_string(), config.total_timeout)
                            }
                        };
                        pw.println(format!(
                            "⌛️ {}: execution timed out after {} at {}",
                            style(test.path.to_string_lossy()).red(),
                            timeout.map_or_else(
                                || "<undef>".to_string(), // this should never happen
                                |t| format_duration(t).to_string()
                            ),
                            location,
                        ));
                        continue;
                    }

                    // ... because test failed with fail_fast enabled
                    ExecutionError::Failed(idx, outputs) => {
                        handle_early_termination(
                            &outputs,
                            &testcases,
                            &mut outcomes,
                            test.path.display().to_string(),
                            escaping.clone(),
                            test.parser_type,
                            &mut count_success,
                            &mut count_failed,
                            &mut count_skipped,
                            |output, testcase| testcase.validate(output),
                        );

                        pw.println(format!(
                            "⚡ {}: stopped at testcase #{} due to fail_fast",
                            style(test.path.to_string_lossy()).red(),
                            idx + 1,
                        ));
                        continue;
                    }

                    // ... because of a final error
                    _ => bail!("failing in {:?}: {}", test.path, err),
                },

                // test execution succeeded
                Ok(outputs) => {
                    if self.debug {
                        debug_testcases(&test.testcases, &test.path, &outputs);
                    }

                    // .. to compare the outputs with testcases and gather that
                    //    outcome for later rendering
                    let (mut failed, mut success) = (0, 0);
                    for (testcase, output) in testcases.into_iter().zip(outputs.into_iter()) {
                        if output.exit_code == ExitStatus::Detached {
                            count_detached += 1;
                            if let Some(ref detached_process) = output.detached_process {
                                kill_detached_process(&pw, detached_process)?;
                            }
                            continue;
                        }

                        let result = testcase.validate(&output);
                        if result.is_err() {
                            failed += 1;
                        } else {
                            success += 1;
                        }
                        outcomes.push(Outcome {
                            location: Some(test.path.display().to_string()),
                            testcase: testcase.clone(),
                            output,
                            escaping: escaping.clone(),
                            format: test.parser_type,
                            result,
                        });
                    }
                    count_failed += failed;
                    count_success += success;
                    let total = failed + success;

                    if failed > 0 {
                        pw.println(format!(
                            "❌ {}: failed {} out of {} testcase{}",
                            style(test.path.to_string_lossy()).red(),
                            style(failed).red().bold(),
                            style(total).bold(),
                            if total == 1 { "" } else { "s" },
                        ));
                    } else if self.verbose {
                        pw.println(format!(
                            "✅ {}: passed {} testcase{}",
                            style(test.path.to_string_lossy()).green(),
                            style(success).green().bold(),
                            if success == 1 { "" } else { "s" },
                        ));
                    }
                }
            }
        }
        pw.println("");
        pw.finish_and_clear();

        // finally render all outcomes of testcase validations
        let renderer: Box<dyn Renderer> = match self.renderer {
            ScrutRenderer::Auto | ScrutRenderer::Pretty => {
                let color_renderer = PrettyColorRenderer {
                    max_surrounding_lines: DEFAULT_SURROUNDING_LINES,
                    absolute_line_numbers: self.absolute_line_numbers,
                    summarize: true,
                    max_multiline_matched_lines: self.max_multiline_matched_lines,
                };
                if !self.global.no_color && console::colors_enabled() {
                    Box::new(color_renderer)
                } else {
                    Box::new(PrettyMonochromeRenderer::new(color_renderer))
                }
            }
            ScrutRenderer::Diff => Box::<DiffRenderer>::default(),
            ScrutRenderer::Json => Box::<JsonRenderer>::default(),
            ScrutRenderer::Yaml => Box::<YamlRenderer>::default(),
        };

        info!(
            success = count_success,
            skipped = count_skipped,
            failed = count_failed,
            detached = count_detached,
        );
        print!("{}", renderer.render(&outcomes.iter().collect::<Vec<_>>())?);

        if count_failed > 0 {
            Err(anyhow!(ValidationFailedError))
        } else {
            Ok(())
        }
    }

    /// Translates command line arguments into a document config, that has only
    /// values set which are provided by the user.
    fn to_document_config(&self) -> DocumentConfig {
        let mut config = DocumentConfig::empty();
        if !self.append_test_file_paths.is_empty() {
            config.append.extend(self.append_test_file_paths.clone());
        }
        if !self.prepend_test_file_paths.is_empty() {
            config.prepend.extend(self.prepend_test_file_paths.clone());
        }

        config.with_defaults_from(&self.global.to_document_config())
    }

    /// Translates command line arguments into a testcase config, that has only
    /// values set which are provided by the user.
    fn to_testcase_config(&self) -> TestCaseConfig {
        self.global.to_testcase_config()
    }
}

/// Helper function to handle early termination cases (timeout, fail_fast).
/// Validates outputs that were collected, marks remaining tests as skipped.
fn handle_early_termination<'a, F>(
    outputs: &[scrut::output::Output],
    testcases: &[&TestCase],
    outcomes: &mut Vec<Outcome>,
    location: String,
    escaping: scrut::escaping::Escaper,
    format: ParserType,
    count_success: &mut usize,
    count_failed: &mut usize,
    count_skipped: &mut usize,
    mut validate_output: F,
) where
    F: FnMut(&scrut::output::Output, &TestCase) -> Result<(), TestCaseError>,
{
    // append outcomes for each testcase that was executed
    outcomes.extend(
        outputs
            .iter()
            .zip(testcases.iter())
            .map(|(output, testcase)| {
                let result = validate_output(output, testcase);
                if result.is_err() {
                    *count_failed += 1;
                } else {
                    *count_success += 1;
                }
                Outcome {
                    location: Some(location.clone()),
                    testcase: (*testcase).clone(),
                    output: output.clone(),
                    escaping: escaping.clone(),
                    format,
                    result,
                }
            }),
    );

    // append outcomes for testcases not executed
    let missing = testcases.len() - outputs.len();
    if missing > 0 {
        outcomes.extend(
            testcases
                .iter()
                .skip(outputs.len())
                .map(|testcase| Outcome {
                    location: Some(location.clone()),
                    testcase: (*testcase).clone(),
                    output: ("", "", None).into(),
                    escaping: escaping.clone(),
                    format,
                    result: Err(TestCaseError::Skipped),
                }),
        );
        *count_skipped += missing;
    }
}

fn prefix_with_directory(prefix: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| prefix.join(path))
        .collect::<Vec<_>>()
}
