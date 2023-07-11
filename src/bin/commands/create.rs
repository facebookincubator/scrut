use std::fs;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use scrut::executors::bash_script_executor::BashScriptExecutor;
use scrut::executors::context::Context as ExecutionContext;
use scrut::executors::execution::Execution;
use scrut::executors::executor::Executor;
use scrut::generators::cram::CramTestCaseGenerator;
use scrut::generators::generator::TestCaseGenerator;
use scrut::generators::markdown::MarkdownTestCaseGenerator;
use scrut::outcome::Outcome;
use scrut::parsers::parser::ParserType;
use scrut::testcase::TestCase;
use tracing::info;

use super::root::GlobalSharedParameters;
use crate::utils::environment::TestEnvironment;
use crate::utils::executorutil::canonical_shell;

/// Create tests from provided shell expression
#[derive(Debug, Parser)]
pub struct Args {
    /// Shell expression THAT WILL BE EXECUTED to automatically create a test from.
    /// Use "-" to read from STDIN.
    #[clap(required = true)]
    shell_expression: Vec<String>,

    /// What kind of test format to create
    #[clap(long, short, default_value = "markdown", value_enum)]
    format: ParserType,

    /// Where to output the created test to (STDOUT is "-")
    #[clap(long, short, default_value = "-")]
    output: String,

    /// What the test is supposed to prove
    #[clap(long, short, default_value = "Command executes successfully")]
    title: String,

    /// Max execution time for the provided shell expression to execute
    #[clap(long, short = 'S', default_value = "900")]
    timeout_seconds: usize,

    #[clap(flatten)]
    global: GlobalSharedParameters,
}

impl Args {
    pub(crate) fn run(&self) -> Result<()> {
        // get timeout for executing of the expression
        if self.timeout_seconds == 0 {
            bail!("timeout must be greater than zero")
        }
        let timeout = Duration::from_secs(self.timeout_seconds as u64);

        // get expression from either STDIN or command line argument(s)
        let expression = if self.shell_expression.len() == 1 && self.shell_expression[0] == "-" {
            std::io::stdin()
                .lock()
                .lines()
                .map(|l| l.context("failed to read STDIN line"))
                .collect::<Result<Vec<_>>>()?
                .join("\n")
        } else {
            self.shell_expression.join(" ")
        };
        let shell_path = canonical_shell(&self.global.shell)?;
        let executor = BashScriptExecutor::new(&shell_path);

        // initialize test environment
        let mut test_environment =
            TestEnvironment::new(&shell_path, self.global.work_directory.as_deref())?;

        // setup test environment ..
        let test_file_path =
            PathBuf::try_from(&test_environment.work_directory)?.join("testfile.tmp");
        let (test_work_directory, environment) =
            test_environment.init_test_file(&test_file_path, self.format == ParserType::Cram)?;
        let environment: &[(&str, &str)] = &environment
            .iter()
            .map(|(k, v)| (k as &str, v as &str))
            .collect::<Vec<_>>();

        // execute the shell expression, get the output
        let outputs = executor
            .execute_all(
                &[&Execution::new(&expression).environment(environment)],
                &ExecutionContext::new()
                    .combine_output(self.global.is_combine_output(None))
                    .crlf_support(self.global.is_keep_output_crlf(None))
                    .directory(Path::new(&test_work_directory))
                    .timeout(Some(timeout)),
            )
            .map_err(|err| anyhow!("{}", err))?;
        assert_eq!(1, outputs.len(), "execution yielded result");

        // generate the testcase
        let generator: Box<dyn TestCaseGenerator> = match self.format {
            ParserType::Cram => Box::<CramTestCaseGenerator>::default(),
            ParserType::Markdown => Box::<MarkdownTestCaseGenerator>::default(),
        };

        // build testcase, run tests to get result
        let testcase = TestCase {
            title: self.title.clone(),
            shell_expression: expression,
            expectations: vec![],
            exit_code: None,
            line_number: 0,
        };
        let result = testcase.validate(&outputs[0]);

        // generate testcase document
        let generated = generator
            .generate_testcases(&[&Outcome {
                location: None,
                output: outputs[0].clone(),
                testcase,
                escaping: self.global.output_escaping(Some(self.format)),
                format: self.format,
                result,
            }])
            .context("generate formatted test")?;

        // write testcase to STDOUT or file
        if self.output == "-" {
            info!("Writing generated test to STDOUT");
            print!("{generated}");
        } else {
            info!(path = &self.output as &str, "Writing generated test");
            fs::write(&self.output, &generated).context("write to output")?;
        }

        Ok(())
    }
}
