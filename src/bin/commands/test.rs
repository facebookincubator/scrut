/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::stdout;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use clap::Parser as ClapParser;
use scrut::config::DocumentConfig;
use scrut::config::TestCaseConfig;
use scrut::executors::context::ContextBuilder;
use scrut::executors::error::ExecutionError;
use scrut::outcome::Outcome;
use scrut::parsers::markdown::DEFAULT_MARKDOWN_LANGUAGES;
use scrut::parsers::parser::ParserType;
use scrut::renderers::diff::DiffRenderer;
use scrut::renderers::pretty::PrettyColorRenderer;
use scrut::renderers::pretty::PrettyMonochromeRenderer;
use scrut::renderers::pretty::DEFAULT_SURROUNDING_LINES;
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
use crate::utils::canonical_shell;
use crate::utils::debug_testcases;
use crate::utils::make_executor;
use crate::utils::FileParser;
use crate::utils::TestEnvironment;

#[derive(Debug, Clone)]
pub struct ValidationFailedError;

impl Display for ValidationFailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "validation failed")
    }
}

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
    #[clap(long, default_value = "*.{md,markdown}")]
    match_markdown: String,

    /// Per default colo(u)r output is enabled on TTYs when the `diff` renderer
    /// is used. This flag disables colo(u)r output in that case
    #[clap(long, alias = "no-colour")]
    no_color: bool,

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
        let mut has_failed = false;
        let mut outcomes = vec![];
        let (mut count_success, mut count_skipped, mut count_failed, mut count_detached) =
            (0, 0, 0, 0);

        // load configuration from command line
        let document_config = self.to_document_config();
        let testcase_config = self.to_testcase_config();
        let current_directory = std::env::current_dir().context("get current directory")?;

        for mut test in tests {
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

            // run all testcases from the file and gather output ..
            let outputs = executor.execute_all(
                testcases.as_slice(),
                &ContextBuilder::default()
                    .work_directory(PathBuf::from(&test_work_directory))
                    .temp_directory(test_environment.tmp_directory.as_path_buf())
                    .config(config)
                    .build()
                    .context("failed to build execution context")?,
            );
            match outputs {
                // test execution failed
                Err(err) => match err {
                    // .. because test was skipped ..
                    ExecutionError::Skipped(_) => {
                        count_skipped += 1;
                        debug!("Received skip code -> skipping tests");
                        outcomes.extend(testcases.into_iter().map(|testcase| Outcome {
                            location: Some(test.path.display().to_string()),
                            testcase: testcase.clone(),
                            output: ("", "", None).into(),
                            escaping: self.global.output_escaping(Some(test.parser_type)),
                            format: test.parser_type,
                            result: Err(TestCaseError::Skipped),
                        }));
                        continue;
                    }

                    // TODO: continue on ExecutionError::Timeout, but warn! or error!

                    // because of a final error
                    _ => bail!("failing in {:?}: {}", test.path, err),
                },

                // test execution succeeded
                Ok(outputs) => {
                    if self.debug {
                        debug_testcases(&test.testcases, &test.path, &outputs);
                    }

                    let testcase_count = testcases.len();
                    let expected_testcases = testcases
                        .into_iter()
                        .filter(|t| !t.config.detached.unwrap_or(false))
                        .collect::<Vec<_>>();
                    count_detached += testcase_count - expected_testcases.len();

                    // this should not happen: different amount of outputs than executed testcases
                    if outputs.len() != expected_testcases.len() {
                        bail!(
                            "expected {} outputs from execution, but got {}",
                            expected_testcases.len(),
                            outputs.len()
                        )
                    }

                    // .. to compare the outputs with testcases and gather that
                    //    outcome for later rendering
                    for (testcase, output) in
                        expected_testcases.into_iter().zip(outputs.into_iter())
                    {
                        let result = testcase.validate(&output);
                        if result.is_err() {
                            count_failed += 1;
                            has_failed = true
                        } else {
                            count_success += 1;
                        }
                        outcomes.push(Outcome {
                            location: Some(test.path.display().to_string()),
                            testcase: testcase.clone(),
                            output,
                            escaping: self.global.output_escaping(Some(test.parser_type)),
                            format: test.parser_type,
                            result,
                        });
                    }
                }
            }
        }

        // finally render all outcomes of testcase validations
        let renderer: Box<dyn Renderer> = match self.renderer {
            ScrutRenderer::Auto | ScrutRenderer::Pretty => {
                let color_renderer = PrettyColorRenderer {
                    max_surrounding_lines: DEFAULT_SURROUNDING_LINES,
                    absolute_line_numbers: self.absolute_line_numbers,
                    summarize: true,
                };
                if stdout().is_terminal() && !self.no_color {
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

        if has_failed {
            Err(anyhow!(ValidationFailedError))
        } else {
            info!("Validation succeeded");
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

fn prefix_with_directory(prefix: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| prefix.join(path))
        .collect::<Vec<_>>()
}
