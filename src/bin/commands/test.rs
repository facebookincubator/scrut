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
use scrut::executors::context::ContextBuilder;
use scrut::executors::error::ExecutionError;
use scrut::executors::execution::Execution;
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
use scrut::testcase::TestCaseError;
use tracing::debug;
use tracing::debug_span;
use tracing::info;

use super::root::GlobalSharedParameters;
use super::root::ScrutRenderer;
use crate::utils::debugutil;
use crate::utils::environment::TestEnvironment;
use crate::utils::executorutil;
use crate::utils::parserutil::ParsedTestFile;
use crate::utils::parserutil::{self};

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
    #[clap(long, short = 'L', default_values = DEFAULT_MARKDOWN_LANGUAGES, num_args = 1..)]
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

    /// For sequential: Timeout in seconds for whole execution. Use 0 for unlimited
    #[clap(long, short = 'S', default_value = "900")]
    timeout_seconds: usize,

    #[clap(flatten)]
    global: GlobalSharedParameters,
}

impl Args {
    pub(crate) fn run(&self) -> Result<()> {
        // init parser and determine suffices to look for
        let (parser_generator, parser_acceptor) = parserutil::make_parser_generator(
            &self.match_cram,
            &self.match_markdown,
            &self
                .markdown_languages
                .iter()
                .map(|s| s as &str)
                .collect::<Vec<_>>(),
        )?;

        let tests = parserutil::parse_test_files(
            "test",
            &self
                .test_file_paths
                .iter()
                .map(|p| p as &Path)
                .collect::<Vec<_>>(),
            &parser_generator,
            &parser_acceptor,
            self.global.cram_compat,
        )?;

        let prepend_tests = parserutil::parse_test_files(
            "prepend test",
            &self
                .prepend_test_file_paths
                .iter()
                .map(|p| p as &Path)
                .collect::<Vec<_>>(),
            &parser_generator,
            &parser_acceptor,
            self.global.cram_compat,
        )?;

        let append_tests = parserutil::parse_test_files(
            "append test",
            &self
                .append_test_file_paths
                .iter()
                .map(|p| p as &Path)
                .collect::<Vec<_>>(),
            &parser_generator,
            &parser_acceptor,
            self.global.cram_compat,
        )?;

        self.run_all(
            &tests.iter().collect::<Vec<_>>(),
            &prepend_tests.iter().collect::<Vec<_>>(),
            &append_tests.iter().collect::<Vec<_>>(),
        )
    }

    fn run_all(
        &self,
        tests: &[&ParsedTestFile],
        prepend_tests: &[&ParsedTestFile],
        append_tests: &[&ParsedTestFile],
    ) -> Result<()> {
        let mut has_failed = false;
        let mut outcomes = vec![];

        let mut test_environment =
            TestEnvironment::new(&self.global.shell, self.global.work_directory.as_deref())?;
        debug!(
            "running {} test files in {:?}",
            tests.len(),
            test_environment
        );

        let (mut count_success, mut count_skipped, mut count_failed) = (0, 0, 0);

        for test in tests {
            let span = debug_span!("test", path = %&test.path.display());
            let _s = span.enter();

            // setup test file environment ..
            let cram_compat = test.parser_type == ParserType::Cram || self.global.cram_compat;
            let (test_work_directory, env_vars) =
                test_environment.init_test_file(&test.path, cram_compat)?;
            let env_vars = env_vars
                .iter()
                .map(|(k, v)| (k as &str, v as &str))
                .collect::<Vec<_>>();

            // extract test cases from content ..
            debug!(
                format = %&test.parser_type,
                num_cases = &test.testcases.len(),
                "running tests",
            );

            // gather executions from prepended, test file and appended
            let mut testcases = prepend_tests
                .iter()
                .flat_map(|test| test.testcases.clone())
                .collect::<Vec<_>>();
            testcases.extend(test.testcases.clone());
            testcases.extend(
                append_tests
                    .iter()
                    .flat_map(|test| test.testcases.clone())
                    .collect::<Vec<_>>(),
            );
            let executions = testcases
                .iter()
                .map(|testcase| Execution::new(&testcase.shell_expression).environment(&env_vars))
                .collect::<Vec<_>>();

            let (timeout, executor) =
                executorutil::make_executor(&self.global.shell, self.timeout_seconds, cram_compat)?;

            // run test cases and gather output ..
            let outputs = executor.execute_all(
                &executions.iter().collect::<Vec<_>>(),
                &ContextBuilder::default()
                    .combine_output(self.global.is_combine_output(Some(test.parser_type)))
                    .crlf_support(self.global.is_keep_output_crlf(Some(test.parser_type)))
                    .work_directory(Some(PathBuf::from(&test_work_directory)))
                    .temp_directory(Some(test_environment.tmp_directory.as_path_buf()))
                    .timeout(timeout)
                    .build()
                    .context("failed to build execution context")?,
            );
            match outputs {
                // test execution failed
                Err(err) => match err {
                    ExecutionError::Skipped(_) => {
                        count_skipped += 1;
                        debug!("Received skip code -> skipping tests");
                        outcomes.extend(testcases.into_iter().map(|testcase| Outcome {
                            location: Some(test.path.display().to_string()),
                            testcase,
                            output: ("", "", None).into(),
                            escaping: self.global.output_escaping(Some(test.parser_type)),
                            format: test.parser_type,
                            result: Err(TestCaseError::Skipped),
                        }));
                        continue;
                    }

                    // because of a final error
                    _ => bail!("failing in {:?}: {}", test.path, err),
                },

                // test execution succeeded
                Ok(outputs) => {
                    if self.debug {
                        debugutil::debug_testcases(&test.testcases, &test.path, &outputs);
                    }

                    if outputs.len() != testcases.len() {
                        bail!(
                            "expected {} outputs from execution, but got {}",
                            testcases.len(),
                            outputs.len()
                        )
                    }
                    debug!("processing outputs");

                    // .. to compare the outputs with testcases and gather that
                    //    outcome for later rendering
                    for (testcase, output) in testcases.into_iter().zip(outputs.into_iter()) {
                        let result = testcase.validate(&output);
                        if result.is_err() {
                            count_failed += 1;
                            has_failed = true
                        } else {
                            count_success += 1;
                        }
                        outcomes.push(Outcome {
                            location: Some(test.path.display().to_string()),
                            testcase,
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
                if stdout().is_terminal() && !self.no_color {
                    Box::new(PrettyColorRenderer::new(
                        DEFAULT_SURROUNDING_LINES,
                        self.absolute_line_numbers,
                    ))
                } else {
                    Box::new(PrettyMonochromeRenderer::new(
                        DEFAULT_SURROUNDING_LINES,
                        self.absolute_line_numbers,
                    ))
                }
            }
            ScrutRenderer::Diff => Box::<DiffRenderer>::default(),
            ScrutRenderer::Json => Box::<JsonRenderer>::default(),
            ScrutRenderer::Yaml => Box::<YamlRenderer>::default(),
        };
        info!(
            success = count_success,
            skipped = count_skipped,
            failed = count_failed
        );
        print!("{}", renderer.render(&outcomes.iter().collect::<Vec<_>>())?);

        if has_failed {
            Err(anyhow!(ValidationFailedError))
        } else {
            info!("Validation succeeded");
            Ok(())
        }
    }
}
