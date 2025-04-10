/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use scrut::config::DocumentConfig;
use scrut::config::TestCaseConfig;
use scrut::executors::bash_script_executor::BashScriptExecutor;
use scrut::executors::context::ContextBuilder;
use scrut::executors::executor::Executor;
use scrut::generators::cram::CramTestCaseGenerator;
use scrut::generators::generator::TestCaseGenerator;
use scrut::generators::markdown::MarkdownTestCaseGenerator;
use scrut::outcome::Outcome;
use scrut::parsers::parser::ParserType;
use scrut::testcase::TestCase;
use tracing::info;

use super::root::GlobalSharedParameters;
use crate::utils::canonical_shell;
use crate::utils::TestEnvironment;

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

    #[clap(flatten)]
    global: GlobalSharedParameters,
}

impl Args {
    pub(crate) fn run(&self) -> Result<()> {
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
        let shell_path = canonical_shell(self.global.shell.as_ref().map(|p| p as &Path))?;
        let executor = BashScriptExecutor::new(&shell_path);

        // initialize test environment
        let mut test_environment = TestEnvironment::new(
            &shell_path,
            self.global.work_directory.as_deref(),
            self.global.keep_temporary_directories,
        )?;

        // setup test environment ..
        let test_file_path = PathBuf::from(&test_environment.work_directory).join("testfile.tmp");
        let (test_work_directory, environment) =
            test_environment.init_test_file(&test_file_path, self.format == ParserType::Cram)?;

        // generate configuration
        let env_vars = BTreeMap::from_iter(environment.iter().map(|(k, v)| (k as &str, v as &str)));
        let (document_config, testcase_config) = if self.format == ParserType::Markdown {
            (
                DocumentConfig::default_markdown(),
                TestCaseConfig::default_markdown(),
            )
        } else {
            (
                DocumentConfig::default_cram(),
                TestCaseConfig::default_cram(),
            )
        };

        // execute the test to get the output
        let testcase_config = testcase_config
            .with_overrides_from(&self.to_testcase_config())
            .with_environment(&env_vars);
        let outputs = executor
            .execute_all(
                &[&TestCase {
                    shell_expression: expression.clone(),
                    config: testcase_config.clone(),
                    ..Default::default()
                }],
                &ContextBuilder::default()
                    .work_directory(PathBuf::from(&test_work_directory))
                    .temp_directory(test_environment.tmp_directory.as_path_buf())
                    .file("testfile.tmp".into())
                    .config(document_config.with_overrides_from(&self.to_document_config()))
                    .build()
                    .context("construct build execution context")?,
            )
            .map_err(|err| anyhow!("{}", err))?;
        assert_eq!(1, outputs.len(), "execution yielded result");

        // build and validate testcase
        let testcase = TestCase {
            title: self.title.clone(),
            shell_expression: expression,
            expectations: vec![],
            exit_code: None,
            line_number: 0,
            config: testcase_config.without_environment(&env_vars),
        };
        let result = testcase.validate(&outputs[0]);

        // generate testcase document
        let generator: Box<dyn TestCaseGenerator> = match self.format {
            ParserType::Cram => Box::<CramTestCaseGenerator>::default(),
            ParserType::Markdown => Box::<MarkdownTestCaseGenerator>::default(),
        };
        let generated = generator
            .generate_testcases(&[&Outcome {
                location: None,
                output: outputs[0].clone(),
                testcase,
                escaping: self.global.output_escaping(Some(self.format)),
                format: self.format,
                result,
            }])
            .context("generate formatted test document content")?;

        // write testcase to STDOUT or file
        if self.output == "-" {
            info!("Writing generated test to STDOUT");
            print!("{generated}");
        } else {
            info!(
                path = &self.output as &str,
                "Writing generated test document"
            );
            fs::write(&self.output, &generated).context("write to output")?;
        }

        Ok(())
    }

    fn to_document_config(&self) -> DocumentConfig {
        self.global.to_document_config()
    }

    fn to_testcase_config(&self) -> TestCaseConfig {
        self.global.to_testcase_config()
    }
}
