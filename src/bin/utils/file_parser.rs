/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use globset::Glob;
use globset::GlobMatcher;
use scrut::config::DocumentConfig;
use scrut::config::TestCaseConfig;
use scrut::expectation::ExpectationMaker;
use scrut::newline::replace_crlf;
use scrut::parsers::cram::CramParser;
use scrut::parsers::cram::DEFAULT_CRAM_INDENTION;
use scrut::parsers::markdown::MarkdownParser;
use scrut::parsers::parser::Parser;
use scrut::parsers::parser::ParserType;
use scrut::rules::glob_cram::CramGlobRule;
use scrut::rules::registry::RuleRegistry;
use scrut::rules::rule::RuleMaker;
use scrut::testcase::TestCase;
use tracing::debug;

/// A utility to parse files or directories using the correct parser [`Parser`] automatically by
/// their file name matching either supported Markdown or Cram file names.
pub struct FileParser<'a> {
    match_cram: GlobMatcher,
    match_markdown: GlobMatcher,
    markdown_languages: &'a [&'a str],
}

impl<'a> FileParser<'a> {
    /// Creata new provide that supports the Markdown / Cram match patterns
    pub fn new(
        match_markdown: &str,
        match_cram: &str,
        markdown_languages: &'a [&'a str],
    ) -> Result<Self> {
        Ok(Self {
            match_markdown: Glob::new(match_markdown)
                .context("create markdown matcher")?
                .compile_matcher(),
            match_cram: Glob::new(match_cram)
                .context("create cram matcher")?
                .compile_matcher(),
            markdown_languages,
        })
    }

    /// Parses all provided paths recursively and retuns all found files with test cases
    pub fn find_and_parse(
        &self,
        name: &str,
        paths: &[&Path],
        cram_compat: bool,
    ) -> Result<Vec<ParsedTestFile>> {
        let contents = self
            .find_all_test_files(paths)
            .with_context(|| format!("read contents from {} file path(s)", name))?;
        let mut result = vec![];
        for (test_file_path, test_file_content) in contents {
            let (parser_type, parser) = self.parser(&test_file_path, cram_compat)?;
            let (config, testcases) = parser.parse(&test_file_content).with_context(|| {
                format!(
                    "parse {} from {:?} with {} parser",
                    name, &test_file_path, parser_type
                )
            })?;
            result.push(ParsedTestFile {
                path: Path::new(&test_file_path).into(),
                content: test_file_content,
                parser_type,
                testcases,
                config,
            });
        }

        Ok(result)
    }

    /// Returns the appropiately configured document [`Parser`]
    fn parser(&self, path: &Path, cram_compat: bool) -> Result<(ParserType, Box<dyn Parser>)> {
        if self.match_markdown.is_match(path) {
            Ok((
                ParserType::Markdown,
                Box::new(MarkdownParser::new(
                    make_expectation_maker(cram_compat),
                    self.markdown_languages,
                    if cram_compat {
                        Some(TestCaseConfig::default_cram())
                    } else {
                        None
                    },
                )),
            ))
        } else if self.match_cram.is_match(path) {
            Ok((
                ParserType::Cram,
                Box::new(CramParser::new(
                    make_expectation_maker(true),
                    DEFAULT_CRAM_INDENTION,
                )),
            ))
        } else {
            Err(anyhow!("no parser found that matches {:?}", path))
        }
    }

    /// Returns true if the provided path matches either the Markdown or the cram file pattern
    fn accept<P: AsRef<Path>>(&self, path: P) -> bool {
        self.match_markdown.is_match(path.as_ref()) || self.match_cram.is_match(path.as_ref())
    }

    /// Returns all test files and their content that are
    fn find_all_test_files<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Vec<(PathBuf, String)>> {
        let mut result = vec![];
        for path in paths {
            if !path.as_ref().exists() {
                bail!("path `{}` does not exist", path.as_ref().display())
            }
            let contents = self
                .read_test_contents(path)
                .with_context(|| format!("scan provided path {}", path.as_ref().display()))?;
            result.extend(contents)
        }
        Ok(result)
    }

    /// Reads test file (or directories, depth-first) at provided path and returns their contents
    fn read_test_contents<P: AsRef<Path>>(&self, path: P) -> Result<Vec<(PathBuf, String)>> {
        let mut result = vec![];

        let attrs = fs::metadata(path.as_ref()).context("read metadata from path")?;
        if attrs.is_dir() {
            let paths = fs::read_dir(path).context("list tests in directory")?;
            for entry in paths {
                let path = entry?.path();
                let sub = self.read_test_contents(&path)?;
                result.extend(sub);
            }
        } else if self.accept(path.as_ref()) {
            let name = path.as_ref().into();
            let contents = read_file(path)?;
            result.push((name, contents));
        }
        Ok(result)
    }
}

fn make_expectation_maker(cram_compat: bool) -> Arc<ExpectationMaker> {
    let mut registry = RuleRegistry::default();

    // override glob rules for cram compatibility mode
    if cram_compat {
        registry.register(CramGlobRule::make, &["glob", "gl"]);
    }

    Arc::new(ExpectationMaker::new(registry))
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    debug!(test_file = %path.as_ref().display(), "reading test file");
    let contents = fs::read(&path).context("read contents from file")?;
    let contents = replace_crlf(&contents[..]);
    String::from_utf8(contents.into()).with_context(|| {
        format!(
            "content file `{}` is not utf-8 encoded",
            path.as_ref().display()
        )
    })
}

/// The parsed instances of a test file that [`FileParser`] creates
pub struct ParsedTestFile {
    pub path: PathBuf,
    pub content: String,
    pub parser_type: ParserType,
    pub testcases: Vec<TestCase>,
    pub config: DocumentConfig,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::FileParser;

    #[test]
    fn test_make_parser_generator() {
        let tests = vec![("file.t", "cram"), ("file.md", "markdown")];

        let provider =
            FileParser::new("*.md", "*.t", &["foo", "bar"]).expect("create parser provider");

        for (file_name, expect) in tests {
            assert!(
                provider.accept(Path::new(file_name)),
                "accept file `{}`",
                file_name,
            );

            let (parser_type, _) = provider
                .parser(Path::new(file_name), false)
                .expect("generate parser");
            assert_eq!(expect, &format!("{}", parser_type));
        }
    }
}
