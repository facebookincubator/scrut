/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use tracing::trace;

use super::context::Context as ExecutionContext;
use super::runner::Runner;
use super::stateful_executor::StatefulExecutorRunnerGenerator;
use super::subprocess_runner::SubprocessRunner;
use crate::output::Output;
use crate::testcase::TestCase;

#[doc = include_str!("./bash_runner.excluded_variables.md")]
pub const BASH_EXCLUDED_VARIABLES: &[&str] = &[
    // variables from Scrut internals
    "__SCRUT_DECLARE_VARS_CMD",
    "__SCRUT_TEMP_STATE_PATH",
    // variables set by scrut in every execution
    "SCRUT_TEST",
    // variables from `man bash`
    "BASHOPTS",
    "BASH_ALIASES",
    "BASH_ARGC",
    "BASH_ARGV",
    "BASH_ARGV0",
    "BASH_CMDS",
    "BASH_COMMAND",
    "BASH_EXECUTION_STRING",
    "BASH_LINENO",
    "BASH_REMATCH",
    "BASH_SOURCE",
    "BASH_SUBSHELL",
    "BASH_VERSINFO",
    "COPROC",
    "DIRSTACK",
    "EUID",
    "FUNCNAME",
    "LINENO",
    "PPID",
    "SHELLOPTS",
    "UID",
];

const BASH_TEMPLATE: &str = include_str!("bash_runner.template");

/// A [`Runner`], that is intended to run a series of contextual related [`Execution`]s, which
/// that ought to share the same environmental context (environment variables, shell
/// variables, shopt, set, functions and aliases).
///
/// It must be initiated with an existing state directory path.
/// After each execution the runner dumps the environmental context in the `state` file of the
/// state directory.
/// Before each execution the runner loads (`source`) the file `state` in the state directory, if
/// it is present.
/// Hence multiple subsequent executions share a consistent environmental context virtually as if
/// they would have been executed from within the same parent bash process (or as close as that is
/// possible without actually running in the same process).
///
/// This Runner is not concurrency-safe (the shared state directory with the `state` file mandates
/// sequential, isolated execution).
///
/// Underlying the [`ThreadedRunner`] is used, so timeout constraints are fully supported.
#[derive(Clone)]
pub struct BashRunner {
    pub shell: PathBuf,
    pub state_directory: PathBuf,
}

impl BashRunner {
    pub fn new(shell: &Path, state_directory: &Path) -> Self {
        Self {
            shell: shell.to_owned(),
            state_directory: state_directory.to_owned(),
        }
    }

    pub fn stateful_generator(shell: &Path) -> StatefulExecutorRunnerGenerator {
        let shell = shell.to_owned();
        Box::new(move |state_directory: &Path| -> Box<dyn Runner> {
            let shell_instance = Self {
                shell: shell.to_owned(),
                state_directory: state_directory.to_owned(),
            };
            Box::new(shell_instance) as Box<dyn Runner>
        })
    }
}

impl Runner for BashRunner {
    fn run(&self, name: &str, testcase: &TestCase, context: &ExecutionContext) -> Result<Output> {
        let shell = self.shell.to_owned();

        // render the bash script
        let state_directory_str = self.state_directory.to_string_lossy();
        let expression = BASH_TEMPLATE
            .replace("{state_directory}", &state_directory_str)
            .replace("{name}", name)
            .replace("{shell_expression}", &testcase.shell_expression)
            .replace("{excluded_variables}", &BASH_EXCLUDED_VARIABLES.join("|"))
            .replace(
                "{persist_state}",
                if testcase.config.detached.unwrap_or(false) {
                    "0"
                } else {
                    "1"
                },
            );
        trace!("compiled expression {}", &expression);

        let mut testcase = testcase.clone();
        testcase.shell_expression = expression;

        SubprocessRunner(shell).run(name, &testcase, context)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::BashRunner;
    use super::Runner;
    use crate::executors::DEFAULT_SHELL;
    use crate::executors::context::Context as ExecutionContext;
    use crate::output::Output;
    use crate::testcase::TestCase;

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_with_timeout_captures_stdout_and_stderr() {
        let temp_dir = TempDir::with_prefix("runner.").expect("create temporary directory");
        let output = BashRunner {
            shell: DEFAULT_SHELL.to_owned(),
            state_directory: temp_dir.path().into(),
        }
        .run(
            "name",
            &TestCase::from_expression("echo OK1 && ( 1>&2 echo OK2 )"),
            &ExecutionContext::new_for_test(),
        )
        .expect("execute without error");
        let expect: Output = ("OK1\n", "OK2\n").into();
        assert_eq!(expect, output);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_captures_non_printable_characters() {
        let temp_dir = TempDir::with_prefix("runner.").expect("create temporary directory");
        let output = BashRunner {
            shell: DEFAULT_SHELL.to_owned(),
            state_directory: temp_dir.path().into(),
        }
        .run(
            "name",
            &TestCase::from_expression("echo -e \"🦀\r\n😊\""),
            &ExecutionContext::new_for_test(),
        )
        .expect("execute without error");

        let expect: Output = ("🦀\n😊\n", "").into();
        assert_eq!(expect, output);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_execute_with_timeout_captures_exit_code() {
        let temp_dir = TempDir::with_prefix("runner.").expect("create temporary directory");
        let output = BashRunner {
            shell: DEFAULT_SHELL.to_owned(),
            state_directory: temp_dir.path().into(),
        }
        .run(
            "name",
            &TestCase::from_expression("( exit 123 )"),
            &ExecutionContext::new_for_test(),
        )
        .expect("execute without error");

        let expect: Output = ("", "", Some(123)).into();
        assert_eq!(expect, output);
    }

    #[test]
    fn test_execute_persists_state_file_in_state_directory() {
        let temp_dir = TempDir::with_prefix("runner.").expect("create temporary directory");
        let _ = BashRunner {
            shell: DEFAULT_SHELL.to_owned(),
            state_directory: temp_dir.path().into(),
        }
        .run(
            "name",
            &TestCase::from_expression("true"),
            &ExecutionContext::new_for_test(),
        )
        .expect("execute without error");

        let state_file = temp_dir.path().join("state");
        assert!(
            state_file.exists(),
            "state file was created during execution"
        );
    }

    #[test]
    fn test_detached_execute_does_not_persist_state_file_in_state_directory() {
        let temp_dir = TempDir::with_prefix("runner.").expect("create temporary directory");
        let mut testcase = TestCase::from_expression("true");
        testcase.config.detached = Some(true);
        let context = ExecutionContext::new_for_test();
        let _ = BashRunner {
            shell: DEFAULT_SHELL.to_owned(),
            state_directory: temp_dir.path().into(),
        }
        .run("name", &testcase, &context)
        .expect("execute without error");

        // wait until after execution execution
        std::thread::sleep(std::time::Duration::from_millis(300));

        let state_file = temp_dir.path().join("state");
        assert!(
            !state_file.exists(),
            "state file was not created during execution"
        );
    }
}
