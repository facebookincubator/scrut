use std::fmt::Debug;
use std::fs;
use std::path::Path;

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
    paths: &[&str],
    accept: &ParserAcceptor,
) -> Result<Vec<(String, String)>> {
    let mut result = vec![];
    for path_str in paths {
        let path = Path::new(path_str);
        if !path.exists() {
            bail!("path `{:?}` does not exist", path)
        }

        let attrs = fs::metadata(path).context("read metadata from path")?;
        if attrs.is_dir() {
            let mut sub = read_directory_recursive(path, accept)
                .context(format!("recurse test directory {}", path_str))?;
            result.append(&mut sub);
        } else if accept(path_str) {
            // no suffix check on first level: assume the user knows best
            let contents = read_file(path)?;
            result.push((canonical_output_path(path).context("test file")?, contents));
        }
    }
    Ok(result)
}

/// Iterates directory recursively, reads all files with given suffices and
/// returns them as list of `[(<path>, <content>)]`
fn read_directory_recursive<P>(
    directory: P,
    accept: &ParserAcceptor,
) -> Result<Vec<(String, String)>>
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
        } else {
            let path = path.to_string_lossy().to_string();
            if accept(&path) {
                let contents = read_file(&path)?;
                result.push((path, contents));
            }
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

/// Takes path and returns (<absolute directory>, <file name>)
///
/// The returned directory path contains forward slashes `/` as path separators,
/// even on Windows.
pub(super) fn split_path_abs(path: &Path) -> Result<(String, String)> {
    let mut directory = path.to_path_buf();
    let file = directory.to_owned();
    let file = file
        .file_name()
        .ok_or_else(|| anyhow!("path is not a file"))?
        .to_string_lossy();
    directory.pop();
    Ok((
        file.to_string(),
        canonical_output_path(directory).context("split path")?,
    ))
}

// All paths that Scrut outputs are canonicalized for the current operation system.
// For windows `dunce` is used to assure that Windows NT forms are only used
// if the path length or reserved words demand it.
pub(crate) fn canonical_output_path<P: AsRef<Path> + Debug>(path: P) -> Result<String> {
    let output = dunce::canonicalize(&path)
        .with_context(|| format!("canonicalize path for output {:?}", path))?
        .to_string_lossy()
        .into();
    Ok(output)
}

#[cfg(test)]
mod tests {

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_output_paths_are_unc_canonicalized() {
        use std::path::PathBuf;

        use super::canonical_output_path;

        let tests = &[
            r"C:\baz\bar\foo",
            r"C:\\baz\\bar\\foo",
            r"C:\baz\bar\foo",
            r"\\?\C:\baz\bar\foo",
            r"C:/baz/bar/foo",
            r"C:\baz\bar/foo",
        ];
        for test in tests {
            let path = PathBuf::from(test);
            match to_output_path(path) {
                Ok(output) => assert_eq!(r"C:\baz\bar\foo", &output),
                Err(_) => {
                    assert!(false, "output path {:?} should canonicalize", path)
                }
            }
        }

        let tests = &[r"\\?\C:/baz/bar/foo", r"\\?\C:\baz\bar/foo"];
        for test in tests {
            let path = PathBuf::from(test);
            let result = to_output_path(path);
            assert!(result.is_err())
        }
    }
}
