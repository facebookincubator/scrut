use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use globset::Glob;
use scrut::expectation::ExpectationMaker;
use scrut::parsers::cram::CramParser;
use scrut::parsers::markdown::MarkdownParser;
use scrut::parsers::parser::Parser;
use scrut::parsers::parser::ParserType;
use scrut::rules::glob_cram::CramGlobRule;
use scrut::rules::registry::RuleRegistry;
use scrut::rules::rule::RuleMaker;
use scrut::testcase::TestCase;

pub(crate) type ParserGenerator = Box<dyn Fn(&str, bool) -> Result<(ParserType, Box<dyn Parser>)>>;
pub(crate) type ParserAcceptor = Box<dyn Fn(&str) -> bool>;

pub(crate) fn make_parser_generator<'a>(
    match_cram: &'a str,
    match_markdown: &'a str,
    markdown_languages: &[&str],
) -> Result<(ParserGenerator, ParserAcceptor)> {
    let languages = markdown_languages
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    // innit parser and determine suffices to look for

    let match_cram = Glob::new(match_cram).context("create cram matcher")?;
    let (match_cram, match_cram_copy) =
        (match_cram.compile_matcher(), match_cram.compile_matcher());

    let match_markdown = Glob::new(match_markdown).context("create markdown matcher")?;
    let (match_markdown, match_markdown_copy) = (
        match_markdown.compile_matcher(),
        match_markdown.compile_matcher(),
    );

    Ok((
        Box::new(move |path, cram_compat| {
            if match_markdown_copy.is_match(path) {
                Ok((
                    ParserType::Markdown,
                    Box::new(MarkdownParser::new(
                        make_expectation_maker(cram_compat),
                        &languages.iter().map(|s| s as &str).collect::<Vec<_>>(),
                    )),
                ))
            } else if match_cram_copy.is_match(path) {
                Ok((
                    ParserType::Cram,
                    Box::new(CramParser::default_new(make_expectation_maker(true))),
                ))
            } else {
                Err(anyhow!("no parser found that matches `{}`", path))
            }
        }),
        Box::new(move |path| match_cram.is_match(path) || match_markdown.is_match(path)),
    ))
}

fn make_expectation_maker(cram_compat: bool) -> Arc<ExpectationMaker> {
    let mut registry = RuleRegistry::default();

    // override glob rules for cram compatibility mode
    if cram_compat {
        registry.register(CramGlobRule::make, &["glob", "gl"]);
    }

    Arc::new(ExpectationMaker::new(registry))
}

/// The parsed instances of a test file
pub(crate) struct ParsedTestFile {
    pub(crate) path: PathBuf,
    pub(crate) content: String,
    pub(crate) parser_type: ParserType,
    pub(crate) testcases: Vec<TestCase>,
}

/// Iterates provided list of paths to test files and returns a list of the `ParsedTestFile` instances
pub(crate) fn parse_test_files(
    name: &str,
    paths: &[&str],
    parser_generator: &ParserGenerator,
    parser_acceptor: &ParserAcceptor,
    cram_compat: bool,
) -> Result<Vec<ParsedTestFile>> {
    let contents = super::fsutil::scan_paths_and_read_contents(paths, parser_acceptor)
        .with_context(|| format!("read contents from {} file path(s)", name))?;
    let mut result = vec![];
    for (test_file_path, test_file_content) in contents {
        let (parser_type, parser) = parser_generator(&test_file_path, cram_compat)?;
        let testcases = parser
            .parse(&test_file_content)
            .with_context(|| format!("parse {} from `{}`", name, test_file_path))?;
        result.push(ParsedTestFile {
            path: Path::new(&test_file_path).into(),
            content: test_file_content,
            parser_type,
            testcases,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::make_parser_generator;

    #[test]
    fn test_make_parser_generator() {
        let tests = vec![("file.t", "cram"), ("file.md", "markdown")];

        for (file_name, expect) in tests {
            let (parser_gen, parser_acceptor) =
                make_parser_generator("*.t", "*.md", &["foo", "bar"])
                    .expect("get parser generator");
            assert!(parser_acceptor(file_name), "accept file `{}`", file_name,);

            let (parser_type, _) = parser_gen(file_name, false).expect("generate parser");
            assert_eq!(expect, &format!("{}", parser_type));
        }
    }
}
