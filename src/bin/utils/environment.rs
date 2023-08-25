use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use tempdir::TempDir;
use tracing::debug;

use super::fsutil::split_path_abs;
use super::nameutil::UniqueNamer;
use crate::utils::executorutil::canonical_shell;

/// A directory within a test environment
pub(crate) enum EnvironmentDirectory {
    /// A temporary directory, that will be cleaned up after is is not in use anymore
    Ephemeral(TempDir),

    /// A permanent (user provided) directory, that will not be cleaned up
    Permanent(PathBuf),
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
            EnvironmentDirectory::Permanent(path) => path.clone(),
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
pub(crate) struct TestEnvironment {
    pub(crate) shell: PathBuf,

    /// The base work directory in which tests are being executed. Can be:
    /// 1. A user provided directory in which all test files will be executed
    /// 2. A temporary generated directory, which will be removed/cleaned up after
    ///    all test executions and in which directories per test-file execution
    ///    will be created (default)
    pub(crate) work_directory: EnvironmentDirectory,

    /// A temporary directory, which will be made available to the test as the
    /// `TMPDIR` environment variable, which will be cleaned up after test
    /// execution. Location and name depends on:
    /// 1. If user provided work directory: Will be temporary `temp.xxxx`
    ///    directory under user provided folder
    /// 2. If not user provided: Will be temporary directory `__tmp` at base
    ///    of temporary work directory where also the per-test directories
    ///    are being created in
    pub(crate) tmp_directory: EnvironmentDirectory,

    /// Ensure unique name of per-test-file directories created within work directory
    pub(crate) namer: UniqueNamer,
}

impl TestEnvironment {
    pub(crate) fn new(shell: &Path, provided_work_directory: Option<&Path>) -> Result<Self> {
        let (work_directory, tmp_directory) = if let Some(directory) = provided_work_directory {
            (
                EnvironmentDirectory::Permanent(directory.into()),
                EnvironmentDirectory::Ephemeral(
                    TempDir::new_in(directory, "temp")
                        .context("create temporary tmp directory in given work directory")?,
                ),
            )
        } else {
            let work = TempDir::new("execution").context("create temporary working directory")?;
            let temp_path = work.path().join("__tmp");
            fs::create_dir(&temp_path)
                .context("create tmp directory in temporary work directory")?;
            (
                EnvironmentDirectory::Ephemeral(work),
                EnvironmentDirectory::Permanent(temp_path),
            )
        };
        debug!(
            "test environment work directory `{:?}`, tmp directory `{:?}`",
            &work_directory, &tmp_directory,
        );

        let namer = UniqueNamer::new(&work_directory.as_path_buf());
        let shell = canonical_shell(shell)?;

        Ok(TestEnvironment {
            shell,
            work_directory,
            tmp_directory,
            namer,
        })
    }

    /// Returns a test environment for a specific test file, consisting of
    /// the work directory (which is unique per test file, unless user provided
    /// a work directory) and a set of environment variables
    pub(crate) fn init_test_file(
        &mut self,
        test_file_path: &Path,
        cram_compat: bool,
    ) -> Result<(PathBuf, Vec<(String, String)>)> {
        let (test_file_directory, test_file_name) =
            split_path_abs(test_file_path).with_context(|| {
                format!("split test file path {:?} into components", &test_file_path)
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
        }
        if let EnvironmentDirectory::Ephemeral(ref temp) = self.tmp_directory {
            debug!("cleaning up temporary tmp directory {:?}", temp.path());
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

impl<'a> TestFileEnvironment<'a> {
    fn build_work_directory(&mut self) -> Result<PathBuf> {
        let test_work_directory = match &self.test_environment.work_directory {
            // if within temporary directory: create unique directory in file
            EnvironmentDirectory::Ephemeral(temp) => {
                let mut test_work_directory: PathBuf = temp.path().into();
                test_work_directory
                    .push(self.test_environment.namer.next_name(self.test_file_name));
                if !test_work_directory.exists() {
                    fs::create_dir(&test_work_directory).context("create working directory")?;
                }
                test_work_directory
            }
            EnvironmentDirectory::Permanent(path) => path.into(),
        };
        Ok(test_work_directory)
    }

    fn build_env_vars(&self) -> Result<Vec<(String, String)>> {
        let tmp = String::try_from(&self.test_environment.tmp_directory)?;
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
                String::try_from(&self.test_environment.work_directory)?,
            ));
            env_vars.push(("TMP".to_string(), tmp.clone()));
            env_vars.push(("TEMP".to_string(), tmp));
        }
        Ok(env_vars)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::env;
    use std::path::Path;
    use std::path::PathBuf;

    use anyhow::Context;
    use tempdir::TempDir;

    use super::TestEnvironment;
    use crate::utils::environment::EnvironmentDirectory;

    #[test]
    fn create_temporary_work_directory_when_none_is_provided() {
        let test_env =
            TestEnvironment::new(Path::new("bash"), None).expect("setup test environment");
        assert!(
            matches!(test_env.work_directory, EnvironmentDirectory::Ephemeral(_)),
            "temporary work directory ephemeral"
        );
        assert!(
            matches!(test_env.tmp_directory, EnvironmentDirectory::Permanent(_)),
            "temporary tmp directory is permanent"
        );
        assert!(
            String::try_from(&test_env.tmp_directory)
                .expect("tmp")
                .starts_with(&String::try_from(&test_env.work_directory).expect("work")),
            "temporary directory in created temporary work directory"
        )
    }

    #[test]
    fn use_provided_work_directory_and_created_tmp_within() {
        let sys_temp_dir = env::temp_dir();
        let test_env = TestEnvironment::new(Path::new("bash"), Some(&sys_temp_dir))
            .expect("setup test environment");
        assert!(
            matches!(test_env.work_directory, EnvironmentDirectory::Permanent(_)),
            "temporary work directory permanent"
        );
        assert!(
            matches!(test_env.tmp_directory, EnvironmentDirectory::Ephemeral(_)),
            "temporary tmp directory is ephemeral"
        );
        assert!(
            String::try_from(&test_env.tmp_directory)
                .expect("tmp")
                .starts_with(&sys_temp_dir.to_string_lossy().to_string()),
            "temporary directory in provided work directory"
        )
    }

    #[test]
    fn temporary_work_directory_is_created_and_cleaned_up() {
        let test_env =
            TestEnvironment::new(Path::new("bash"), None).expect("setup test environment");
        let directory = String::try_from(&test_env.work_directory).expect("work directory");
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
        let test_env = TestEnvironment::new(Path::new("bash"), Some(&sys_temp_dir))
            .expect("setup test environment");
        let directory = String::try_from(&test_env.tmp_directory).expect("tmp_directory");
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
    fn test_file_environment_setup() {
        let provided_directory = TempDir::new("provided").expect("create provided temp directory");
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
                TestEnvironment::new(Path::new("bash"), None).expect("setup test environment"),
                true,
            ),
            (
                false,
                TestEnvironment::new(Path::new("bash"), None).expect("setup test environment"),
                false,
            ),
            (
                true,
                TestEnvironment::new(Path::new("bash"), Some(provided_directory_path))
                    .expect("setup test environment"),
                true,
            ),
            (
                true,
                TestEnvironment::new(Path::new("bash"), Some(provided_directory_path))
                    .expect("setup test environment"),
                false,
            ),
        ];

        for (idx, (has_provided_work_dir, test_env, cram_compat)) in tests.iter_mut().enumerate() {
            let test_file_name = format!("some-test-file-{}.md", idx + 1);
            let test_file_path = PathBuf::try_from(&test_env.work_directory)
                .with_context(|| format!("work_directory {:?}", test_env))
                .unwrap()
                .join(&test_file_name);
            /* fs::File::create(&test_file_path)
            .with_context(|| format!("create dummy test file {:?}", test_env))
            .unwrap(); */
            let (work_dir, env_vars) = test_env
                .init_test_file(&test_file_path, *cram_compat)
                .with_context(|| format!("initialize for test file {:?}", test_env))
                .unwrap();
            if *has_provided_work_dir {
                assert!(
                    work_dir.starts_with(provided_directory_path),
                    "test file work directory {:?} in provided work directory {:?}",
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
                    "test file work directory {:?} derived from test file",
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
