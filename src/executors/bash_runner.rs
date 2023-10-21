use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use tracing::trace;

use super::context::Context as ExecutionContext;
use super::execution::Execution;
use super::runner::Runner;
use super::stateful_executor::StatefulExecutorRunnerGenerator;
use super::subprocess_runner::SubprocessRunner;
use crate::output::Output;

#[doc = include_str!("./bash_runner.excluded_variables.md")]
pub const BASH_EXCLUDED_VARIABLES: &[&str] = &[
    // variables from Scrut internals
    "__SCRUT_TEMP_STATE_PATH",
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
    fn run(&self, name: &str, execution: &Execution, context: &ExecutionContext) -> Result<Output> {
        let shell = self.shell.to_owned();

        // render the bash script
        let state_directory_str = self.state_directory.to_string_lossy();
        let expression = BASH_TEMPLATE
            .replace("{state_directory}", &state_directory_str)
            .replace("{name}", name)
            .replace("{shell_expression}", &execution.expression)
            .replace("{excluded_variables}", &BASH_EXCLUDED_VARIABLES.join("|"));
        trace!("compiled expression {}", &expression);

        SubprocessRunner(shell).run(name, &execution.to_owned().expression(&expression), context)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::BashRunner;
    use super::Runner;
    use crate::executors::context::Context as ExecutionContext;
    use crate::executors::execution::Execution;
    use crate::executors::DEFAULT_SHELL;
    use crate::output::Output;

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
            &Execution::new("echo OK1 && ( 1>&2 echo OK2 )"),
            &ExecutionContext::new(),
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
            &Execution::new("echo -e \"🦀\r\n😊\""),
            &ExecutionContext::new(),
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
            &Execution::new("( exit 123 )"),
            &ExecutionContext::new(),
        )
        .expect("execute without error");

        let expect: Output = ("", "", Some(123)).into();
        assert_eq!(expect, output);
    }
}
