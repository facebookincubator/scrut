use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use scrut::newline::replace_crlf;
use tracing::debug;

use super::parserutil::ParserAcceptor;

/// Scans all provides paths, recurses any which is a directory, and returns
/// a list of all files containing tests as a list of `[(<path>, <content>)]`
pub(crate) fn scan_paths_and_read_contents(
    paths: &[&Path],
    accept: &ParserAcceptor,
) -> Result<Vec<(PathBuf, String)>> {
    let mut result = vec![];
    for path in paths {
        if !path.exists() {
            bail!("path `{:?}` does not exist", path)
        }

        let attrs = fs::metadata(path).context("read metadata from path")?;
        if attrs.is_dir() {
            let mut sub = read_directory_recursive(path, accept)
                .context(format!("recurse test directory {:?}", path))?;
            result.append(&mut sub);
        } else if accept(path) {
            // no suffix check on first level: assume the user knows best
            let contents = read_file(path)?;
            result.push((path.into(), contents));
        }
    }
    Ok(result)
}

/// Iterates directory recursively, reads all files with given suffices and
/// returns them as list of `[(<path>, <content>)]`
fn read_directory_recursive<P>(
    directory: P,
    accept: &ParserAcceptor,
) -> Result<Vec<(PathBuf, String)>>
where
    P: AsRef<Path>,
{
    let mut result = vec![];
    let paths = fs::read_dir(directory).context("list tests in directory")?;
    for entry in paths {
        let path = entry?.path();
        let attrs = fs::metadata(&path)?;
        if attrs.is_dir() {
            let mut sub = read_directory_recursive(&path, accept)?;
            result.append(&mut sub);
        } else if accept(&path) {
            let contents = read_file(&path)?;
            result.push((path, contents));
        }
    }
    Ok(result)
}

fn read_file<P: AsRef<Path> + Debug>(path: P) -> Result<String> {
    debug!(test_file = %path.as_ref().display(), "reading test file");
    let contents = fs::read(&path).context("read contents from file")?;
    let contents = replace_crlf(&contents[..]);
    String::from_utf8(contents.into())
        .with_context(|| format!("content file `{:?}` is not utf-8 encoded", path))
}

/// Split given path into file name and base directory
pub(super) fn split_path_abs(path: &Path) -> Result<(PathBuf, PathBuf)> {
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

// All paths that Scrut outputs are canonicalized for the current operation system.
// For windows `dunce` is used to assure that Windows NT forms are only used
// if the path length or reserved words demand it.
pub(crate) fn canonical_path<P: AsRef<Path> + Debug>(path: P) -> Result<PathBuf> {
    Ok(dunce::canonicalize(&path)?)
}
