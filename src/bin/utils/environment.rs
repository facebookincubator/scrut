/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use scrut::executors::DEFAULT_SHELL;
use tempfile::TempDir;
use tracing::debug;

use super::namer::UniqueNamer;

/// A directory within a test environment
pub enum EnvironmentDirectory {
    /// A temporary directory, that will be cleaned up after is is not in use anymore
    Ephemeral(TempDir),

    /// A permanent directory, that will not be cleaned up
    UserProvided(PathBuf),

    /// A temporary directory that is created by Scrut and not removed / cleaned up
    Kept(PathBuf),
}

impl EnvironmentDirectory {
    pub fn as_path_buf(&self) -> PathBuf {
        self.into()
    }
}

impl From<&EnvironmentDirectory> for PathBuf {
    fn from(value: &EnvironmentDirectory) -> Self {
        match value {
            EnvironmentDirectory::Ephemeral(temp) => temp.path().into(),
            EnvironmentDirectory::UserProvided(path) => path.clone(),
            EnvironmentDirectory::Kept(path) => path.clone(),
        }
    }
}

impl From<&EnvironmentDirectory> for String {
    fn from(value: &EnvironmentDirectory) -> Self {
        PathBuf::from(value).to_string_lossy().to_string()
    }
}

impl Debug for EnvironmentDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

/// Encapsulate test directory and environment variables setup
pub struct TestEnvironment {
    pub shell: PathBuf,

    /// The base work directory in which tests are being executed. Can be:
    /// 1. A user provided directory in which all test files will be executed
    /// 2. A temporary generated directory, which will be removed/cleaned up after
    ///    all test executions and in which directories per test-file execution
    ///    will be created (default)
    pub work_directory: EnvironmentDirectory,

    /// A temporary directory, which will be made available to the test as the
    /// `TMPDIR` environment variable, which will be cleaned up after test
    /// execution. Location and name depends on:
    /// 1. If user provided work directory: Will be temporary `temp.xxxx`
    ///    directory under user provided folder
    /// 2. If not user provided: Will be temporary directory `__tmp` at base
    ///    of temporary work directory where also the per-test directories
    ///    are being created in
    pub tmp_directory: EnvironmentDirectory,

    /// Ensure unique name of per-test-file directories created within work directory
    namer: UniqueNamer,
}

impl TestEnvironment {
    pub fn new(
        shell: &Path,
        provided_work_directory: Option<&Path>,
        keep_temporary_directories: bool,
    ) -> Result<Self> {
        let (work_directory, tmp_directory) = if keep_temporary_directories {
            let work_path = TempDir::with_prefix("execution.")
                .context("create temporary working directory")?
                .keep();
            let temp_path = TempDir::with_prefix("temp.")
                .context("create temporary working directory")?
                .keep();
            (
                EnvironmentDirectory::Kept(work_path),
                EnvironmentDirectory::Kept(temp_path),
            )
        } else if let Some(directory) = provided_work_directory {
            (
                EnvironmentDirectory::UserProvided(directory.into()),
                EnvironmentDirectory::Ephemeral(
                    TempDir::with_prefix_in("temp.", directory)
                        .context("create temporary tmp directory in given work directory")?,
                ),
            )
        } else {
            let work =
                TempDir::with_prefix("execution.").context("create temporary working directory")?;
            let temp_path = work.path().join("__tmp");
            fs::create_dir(&temp_path)
                .context("create tmp directory in temporary work directory")?;
            (
                EnvironmentDirectory::Ephemeral(work),
                EnvironmentDirectory::UserProvided(temp_path),
            )
        };
        debug!(
            "test environment work directory `{:?}`, tmp directory `{:?}`",
            &work_directory, &tmp_directory,
        );

        let namer = UniqueNamer::new(&work_directory.as_path_buf());

        Ok(TestEnvironment {
            shell: shell.into(),
            work_directory,
            tmp_directory,
            namer,
        })
    }

    /// Returns a test environment for a specific test file, consisting of
    /// the work directory (which is unique per test file, unless user provided
    /// a work directory) and a set of environment variables
    pub fn init_test_file(
        &mut self,
        test_file_path: &Path,
        cram_compat: bool,
    ) -> Result<(PathBuf, Vec<(String, String)>)> {
        let (test_file_directory, test_file_name) =
            split_path_abs(test_file_path).with_context(|| {
                format!(
                    "split test document file path {:?} into components",
                    &test_file_path
                )
            })?;

        let mut per_file = TestFileEnvironment {
            test_environment: self,
            test_file_name: &test_file_name,
            test_file_directory: &test_file_directory,
            cram_compat,
        };

        Ok((per_file.build_work_directory()?, per_file.build_env_vars()?))
    }
}

impl Debug for TestEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestEnvironment")
            .field("shell", &self.shell)
            .field("work_directory", &self.work_directory)
            .field("tmp_directory", &self.tmp_directory)
            .finish()
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if let EnvironmentDirectory::Ephemeral(ref temp) = self.work_directory {
            debug!("cleaning up temporary work directory {:?}", temp.path());
        } else if let EnvironmentDirectory::Kept(ref temp) = self.work_directory {
            debug!("keeping temporary work directory {:?}", temp);
        }
        if let EnvironmentDirectory::Ephemeral(ref temp) = self.tmp_directory {
            debug!("cleaning up temporary tmp directory {:?}", temp.path());
        } else if let EnvironmentDirectory::Kept(ref temp) = self.tmp_directory {
            debug!("keeping temporary tmp directory {:?}", temp);
        }
    }
}

/// The environment per file, that builds on the [`TestEnvironment`]
struct TestFileEnvironment<'a> {
    test_environment: &'a mut TestEnvironment,
    test_file_name: &'a Path,
    test_file_directory: &'a Path,
    cram_compat: bool,
}

impl TestFileEnvironment<'_> {
    fn build_work_directory(&mut self) -> Result<PathBuf> {
        let test_work_directory: PathBuf = match &self.test_environment.work_directory {
            // if within temporary directory: create unique directory in file
            EnvironmentDirectory::Ephemeral(temp) => create_random_sub_directory(
                temp.path(),
                self.test_file_name,
                &mut self.test_environment.namer,
            )?,
            EnvironmentDirectory::Kept(temp) => create_random_sub_directory(
                temp,
                self.test_file_name,
                &mut self.test_environment.namer,
            )?,
            EnvironmentDirectory::UserProvided(path) => path.into(),
        };
        Ok(test_work_directory)
    }

    fn build_env_vars(&self) -> Result<Vec<(String, String)>> {
        let tmp = String::from(&self.test_environment.tmp_directory);
        let mut env_vars = vec![
            (
                "TESTDIR".to_string(),
                self.test_file_directory.to_string_lossy().to_string(),
            ),
            (
                "TESTFILE".to_string(),
                self.test_file_name.to_string_lossy().to_string(),
            ),
            ("TMPDIR".to_string(), tmp.clone()),
            (
                "TESTSHELL".to_string(),
                self.test_environment.shell.to_string_lossy().to_string(),
            ),
            ("LANG".to_string(), "C".to_string()),
            ("LANGUAGE".to_string(), "C".to_string()),
            ("LC_ALL".to_string(), "C".to_string()),
            ("TZ".to_string(), "GMT".to_string()),
            ("COLUMNS".to_string(), "80".to_string()),
            ("CDPATH".to_string(), "".to_string()),
            ("GREP_OPTIONS".to_string(), "".to_string()),
        ];
        if self.cram_compat {
            env_vars.push((
                "CRAMTMP".to_string(),
                String::from(&self.test_environment.work_directory),
            ));
            env_vars.push(("TMP".to_string(), tmp.clone()));
            env_vars.push(("TEMP".to_string(), tmp));
        }
        Ok(env_vars)
    }
}

fn create_random_sub_directory(
    directory: &Path,
    file_name: &Path,
    namer: &mut UniqueNamer,
) -> Result<PathBuf, anyhow::Error> {
    let mut directory: PathBuf = directory.into();
    directory.push(namer.next_name(file_name));
    if !directory.exists() {
        fs::create_dir(&directory).context("create working directory")?;
    }
    Ok(directory)
}

/// Returns the canonical path to the given shell
pub fn canonical_shell(shell: Option<&Path>) -> Result<PathBuf> {
    let shell = shell.unwrap_or(*DEFAULT_SHELL);
    if shell.components().count() > 1 {
        canonical_path(shell)
    } else {
        canonical_path(
            which::which(shell)
                .with_context(|| format!("guessing path to shell `{}`", shell.display()))?
                .as_path(),
        )
    }
    .context("path to shell")
}

// All paths that Scrut outputs are canonicalized for the current operation system.
// For windows `dunce` is used to assure that Windows NT forms are only used
// if the path length or reserved words demand it.
fn canonical_path<P: AsRef<Path> + Debug>(path: P) -> Result<PathBuf> {
    Ok(dunce::canonicalize(&path)?)
}

/// Split given path into file name and base directory
fn split_path_abs(path: &Path) -> Result<(PathBuf, PathBuf)> {
    let mut directory = path.to_path_buf();
    let file = directory
        .file_name()
        .ok_or_else(|| anyhow!("path is not a file"))?
        .into();
    directory.pop();
    let directory = if directory.to_string_lossy().is_empty() {
        std::env::current_dir().context("split path")?
    } else {
        canonical_path(&directory).context("split path")?
    };
    Ok((directory, file))
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::env;
    use std::path::Path;
    use std::path::PathBuf;

    use anyhow::Context;
    use tempfile::TempDir;

    use super::TestEnvironment;
    use crate::utils::environment::EnvironmentDirectory;

    #[test]
    fn create_temporary_work_directory_when_none_is_provided() {
        let test_env =
            TestEnvironment::new(Path::new("bash"), None, false).expect("setup test environment");
        assert!(
            matches!(test_env.work_directory, EnvironmentDirectory::Ephemeral(_)),
            "temporary work directory ephemeral"
        );
        assert!(
            matches!(
                test_env.tmp_directory,
                EnvironmentDirectory::UserProvided(_)
            ),
            "temporary tmp directory is user-provided"
        );
        assert!(
            String::from(&test_env.tmp_directory)
                .starts_with(&String::from(&test_env.work_directory)),
            "temporary directory in created temporary work directory"
        )
    }

    #[test]
    fn use_provided_work_directory_and_created_tmp_within() {
        let sys_temp_dir = env::temp_dir();
        let test_env = TestEnvironment::new(Path::new("bash"), Some(&sys_temp_dir), false)
            .expect("setup test environment");
        assert!(
            matches!(
                test_env.work_directory,
                EnvironmentDirectory::UserProvided(_)
            ),
            "temporary work directory user-provided"
        );
        assert!(
            matches!(test_env.tmp_directory, EnvironmentDirectory::Ephemeral(_)),
            "temporary tmp directory is ephemeral"
        );
        assert!(
            String::from(&test_env.tmp_directory)
                .starts_with(&sys_temp_dir.to_string_lossy().to_string()),
            "temporary directory in provided work directory"
        )
    }

    #[test]
    fn keep_temporary_directories_if_requested_by_user() {
        let test_env =
            TestEnvironment::new(Path::new("bash"), None, true).expect("setup test environment");
        assert!(
            matches!(test_env.work_directory, EnvironmentDirectory::Kept(_)),
            "temporary work directory kept"
        );
        assert!(
            matches!(test_env.tmp_directory, EnvironmentDirectory::Kept(_)),
            "temporary tmp directory is kept"
        );
        let tmp_directory = String::from(&test_env.tmp_directory);
        let work_directory = String::from(&test_env.work_directory);
        assert!(
            !&tmp_directory.starts_with(&work_directory),
            "tmp directory is not under work directory"
        );
        assert!(
            !&work_directory.starts_with(&tmp_directory),
            "work directory is not under tmp directory"
        );
    }

    #[test]
    fn temporary_work_directory_is_created_and_cleaned_up() {
        let test_env =
            TestEnvironment::new(Path::new("bash"), None, false).expect("setup test environment");
        let directory = String::from(&test_env.work_directory);
        assert!(
            Path::new(&directory).exists(),
            "temporary work directory is created"
        );
        drop(test_env);
        assert!(
            !Path::new(&directory).exists(),
            "temporary work directory is removed"
        );
    }

    #[test]
    fn temporary_tmp_directory_is_created_and_cleaned_up() {
        let sys_temp_dir = env::temp_dir();
        let test_env = TestEnvironment::new(Path::new("bash"), Some(&sys_temp_dir), false)
            .expect("setup test environment");
        let directory = String::from(&test_env.tmp_directory);
        assert!(
            Path::new(&directory).exists(),
            "temporary tmp directory is created"
        );
        drop(test_env);
        assert!(
            !Path::new(&directory).exists(),
            "temporary tmp directory is removed"
        );
    }

    #[test]
    fn kept_temporary_directories_are_created_but_not_cleaned_up() {
        let test_env =
            TestEnvironment::new(Path::new("bash"), None, true).expect("setup test environment");
        let tmp_directory = String::from(&test_env.tmp_directory);
        let work_directory = String::from(&test_env.work_directory);
        assert!(
            Path::new(&tmp_directory).exists(),
            "temporary tmp directory is created"
        );
        assert!(
            Path::new(&work_directory).exists(),
            "temporary work directory is created"
        );
        drop(test_env);
        assert!(
            Path::new(&tmp_directory).exists(),
            "temporary tmp directory is not removed"
        );
        assert!(
            Path::new(&work_directory).exists(),
            "temporary work directory is not removed"
        );
    }

    #[test]
    fn test_file_environment_setup() {
        let provided_directory =
            TempDir::with_prefix("provided.").expect("create provided temp directory");
        let provided_directory_path = provided_directory.path();
        let expected_variables = &[
            "CDPATH",
            "COLUMNS",
            "GREP_OPTIONS",
            "LANG",
            "LANGUAGE",
            "LC_ALL",
            "TESTDIR",
            "TESTFILE",
            "TESTSHELL",
            "TMPDIR",
            "TZ",
        ];
        let expected_variables_cram = &["CRAMTMP", "TEMP", "TMP"];
        let tests = &mut [
            (
                false,
                TestEnvironment::new(Path::new("bash"), None, false)
                    .expect("setup test environment"),
                true,
            ),
            (
                false,
                TestEnvironment::new(Path::new("bash"), None, false)
                    .expect("setup test environment"),
                false,
            ),
            (
                true,
                TestEnvironment::new(Path::new("bash"), Some(provided_directory_path), false)
                    .expect("setup test environment"),
                true,
            ),
            (
                true,
                TestEnvironment::new(Path::new("bash"), Some(provided_directory_path), false)
                    .expect("setup test environment"),
                false,
            ),
        ];

        for (idx, (has_provided_work_dir, test_env, cram_compat)) in tests.iter_mut().enumerate() {
            let test_file_name = format!("some-test-file-{}.md", idx + 1);
            let test_file_path = PathBuf::from(&test_env.work_directory).join(&test_file_name);
            let (work_dir, env_vars) = test_env
                .init_test_file(&test_file_path, *cram_compat)
                .with_context(|| format!("initialize for test document {:?}", test_env))
                .unwrap();
            if *has_provided_work_dir {
                assert!(
                    work_dir.starts_with(provided_directory_path),
                    "test document work directory {:?} in provided work directory {:?}",
                    &work_dir,
                    &provided_directory_path,
                );
            } else {
                let file_name = Path::new(&work_dir)
                    .components()
                    .last()
                    .map(|d| d.as_os_str().to_string_lossy().to_string());
                assert_eq!(
                    file_name,
                    Some(test_file_name),
                    "work directory {:?} is derived from test document file path",
                    &work_dir,
                );
            }
            let mut env_vars = env_vars.into_iter().collect::<HashMap<_, _>>();
            for name in expected_variables {
                assert!(
                    env_vars.contains_key(*name),
                    "{} is defined environment variable",
                    name
                );
                env_vars.remove(*name);
            }
            if *cram_compat {
                for name in expected_variables_cram {
                    assert!(
                        env_vars.contains_key(*name),
                        "{} is defined cram environment variable",
                        name
                    );
                    env_vars.remove(*name);
                }
            }
            assert!(
                env_vars.is_empty(),
                "all defined environment variables are accounted for"
            );
        }
    }
}
