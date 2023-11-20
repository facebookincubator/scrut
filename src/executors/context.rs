use std::path::PathBuf;

use derive_builder::Builder;

use crate::config::DocumentConfig;

/// Context that describes the environment in which one or multiple [`crate::testcase::TestCase`]s are executed in
#[derive(Clone, Default, Debug, PartialEq, Eq, Builder)]
pub struct Context {
    /// Optional cwd path for the execution
    #[builder(default)]
    pub work_directory: Option<PathBuf>,

    /// Optional path for that holds temporary files
    #[builder(default)]
    pub temp_directory: Option<PathBuf>,

    /// The configuration on per-document level
    #[builder(default)]
    pub config: DocumentConfig,
}
