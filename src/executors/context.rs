/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::PathBuf;

use derive_builder::Builder;

use crate::config::DocumentConfig;

/// Context that describes the environment in which one or multiple [`crate::testcase::TestCase`]s are executed in
#[derive(Debug, PartialEq, Eq, Builder)]
pub struct Context {
    /// Optional cwd path for the execution
    pub work_directory: PathBuf,

    /// Optional path for that holds temporary files
    pub temp_directory: PathBuf,

    /// The configuration on per-document level
    #[builder(default)]
    pub config: DocumentConfig,
}

#[cfg(test)]
impl Context {
    /// Create a new context that is used by test. Both temporary and working directory
    /// will be automatically removed when the context is dropped
    pub fn new_for_test() -> Self {
        Self::new_for_test_with_config(Default::default())
    }

    #[cfg(test)]
    pub fn new_for_test_with_config(config: DocumentConfig) -> Self {
        Self {
            work_directory: test::create_testing_directory(),
            temp_directory: test::create_testing_directory(),
            config,
        }
    }
}

// Clean-up of the testing temp and work directories needs to be explicitly implemented
// because [`Context`] needs them to be [`PathBuf`] (instead of [`tempfile::TempDir`])
// as they may contain paths which explicitly must not be cleaned up (e.g. `--working-directory`).
// The [`Drop`] implementation here is only implemented for `#[cfg(test)]` and only
// acts on directories that were created in [`Context::new_for_test`]
#[cfg(test)]
impl Drop for Context {
    fn drop(&mut self) {
        for (name, directory) in [
            ("temp", &self.temp_directory),
            ("work", &self.work_directory),
        ] {
            if directory
                .to_string_lossy()
                .contains(test::TESTING_PATH_PREFIX)
            {
                std::fs::remove_dir_all(directory).unwrap_or_else(|err| {
                    panic!(
                        "failed to clean up testing {name} directory recursively in \"{}\": {}",
                        directory.display(),
                        err
                    )
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use tempfile::tempdir;
    use tempfile::Builder;

    use super::Context;

    /// Prefix that is added to any temp or work directory created by [`Context::new_for_test`],
    /// so that such directories can be identified and cleaned up.
    pub(super) const TESTING_PATH_PREFIX: &str = "scrut-selftest-temp-directory";

    /// Creates a new temporary directory that can be identified as created for
    /// testing, and that is not automatically cleaned up by the `tempfile` crate.
    pub(super) fn create_testing_directory() -> PathBuf {
        Builder::new()
            .prefix(TESTING_PATH_PREFIX)
            .tempdir()
            .expect("create testing working directory for context")
            .into_path()
    }

    #[test]
    fn test_testing_context_creates_directories() {
        let context = Context::new_for_test();
        assert!(context.temp_directory.exists(), "temp directory is created");
        assert!(
            context
                .temp_directory
                .to_string_lossy()
                .contains(TESTING_PATH_PREFIX),
            "temp directory has identifying prefix"
        );
        assert!(context.work_directory.exists(), "work directory is created");
        assert!(
            context
                .work_directory
                .to_string_lossy()
                .contains(TESTING_PATH_PREFIX),
            "work directory has identifying prefix"
        );
    }

    #[test]
    fn test_testing_context_drop_removes_directories_it_created() {
        let context = Context::new_for_test();
        let temp_directory = context.temp_directory.clone();
        let work_directory = context.work_directory.clone();
        assert!(temp_directory.exists(), "temp directory is created");
        assert!(work_directory.exists(), "work directory is created");
        drop(context);
        assert!(!temp_directory.exists(), "temp directory is cleaned up");
        assert!(!work_directory.exists(), "work directory is cleaned up");
    }

    #[test]
    fn test_testing_context_drop_does_not_remove_directories_it_did_not_create() {
        let temp_directory = tempdir().expect("crate temp directory");
        let work_directory = tempdir().expect("crate work directory");
        let context = Context {
            temp_directory: temp_directory.path().to_path_buf(),
            work_directory: work_directory.path().to_path_buf(),
            config: Default::default(),
        };

        assert!(temp_directory.path().exists(), "temp directory is created");
        assert!(work_directory.path().exists(), "work directory is created");
        drop(context);
        assert!(
            temp_directory.path().exists(),
            "temp directory is NOT cleaned up"
        );
        assert!(
            work_directory.path().exists(),
            "work directory is NOT cleaned up"
        );
    }
}
