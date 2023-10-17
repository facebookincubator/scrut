use std::str::Lines;
use std::sync::Arc;

use anyhow::Result;
use regex::Regex;
use tracing::debug;

use super::line_parser::is_comment;
use super::parser::Parser;
use crate::expectation::ExpectationMaker;
use crate::parsers::line_parser::LineParser;
use crate::testcase::TestCase;

lazy_static! {
    static ref PARAGRAPH_START: Regex =
        Regex::new(r"^\p{L}+").expect("paragraph start expression must compile");
    static ref HEADER_LINE: Regex =
        Regex::new(r"^(#+\s+)(.+)$").expect("header start expression must compile");
}

pub const DEFAULT_MARKDOWN_LANGUAGES: &[&str] = &["scrut", "testcase"];

/// A parser for Cram `.t` files, which reads [`crate::testcase::TestCase`]s
/// that are encoded in the form:
///
/// ````markdown
/// A title
///
/// ```
/// $ command
/// expectation
/// ```
/// ````
pub struct MarkdownParser {
    expectation_maker: Arc<ExpectationMaker>,
    languages: Vec<String>,
}

impl MarkdownParser {
    pub fn new(expectation_maker: Arc<ExpectationMaker>, languages: &[&str]) -> Self {
        Self {
            expectation_maker,
            languages: languages.iter().map(|lang| lang.to_string()).collect(),
        }
    }

    pub fn default_new(expectation_maker: Arc<ExpectationMaker>) -> Self {
        Self::new(expectation_maker, DEFAULT_MARKDOWN_LANGUAGES)
    }
}

impl Parser for MarkdownParser {
    /// See [`super::parser::Parser::parse`]
    fn parse(&self, text: &str) -> Result<Vec<TestCase>> {
        debug!(
            "parsing markdown file, looking for code blocks with language `{}`",
            &self.languages.join("` or `")
        );

        let languages: &[&str] = &self.languages.iter().map(|s| s as &str).collect::<Vec<_>>();
        let iterator = MarkdownIterator::new(languages, text.lines());
        let mut line_parser = LineParser::new(self.expectation_maker.clone(), false);
        let mut title_paragraph = vec![];

        for token in iterator {
            match token {
                MarkdownToken::Line(_, line) => {
                    if let Some((_, title)) = extract_title(&line) {
                        title_paragraph.push(title);
                        line_parser.set_testcase_title(&title_paragraph.join("\n"));
                    } else if !title_paragraph.is_empty() {
                        title_paragraph.clear();
                    }
                }
                MarkdownToken::CodeBlock {
                    code_lines,
                    comment_lines: _,
                    language: _,
                } => {
                    for (index, line) in &code_lines {
                        line_parser.add_testcase_body(line, *index)?;
                    }
                    line_parser.end_testcase(code_lines[code_lines.len() - 1].0)?;
                    title_paragraph.clear();
                }
            }
        }
        debug!(
            "found {} testcases in markdown file",
            line_parser.testcases.len()
        );

        Ok(line_parser.testcases.clone())
    }
}

/// An element of a Markdown document that we care about knowing
#[derive(Debug)]
pub(crate) enum MarkdownToken {
    /// An arbitrary line; basically any line of markdown we do not care about
    Line(usize, String),

    /// The parsed contents of a code block within backticks
    CodeBlock {
        code_lines: Vec<(usize, String)>,
        comment_lines: Vec<(usize, String)>,
        language: String,
    },
}

/// An iterator that parses Markdown documents in lines and code-blocks
pub(crate) struct MarkdownIterator<'a> {
    languages: &'a [&'a str],
    document_lines: Lines<'a>,

    // state
    line_index: usize,
}

impl<'a> MarkdownIterator<'a> {
    pub fn new(languages: &'a [&'a str], document_lines: Lines<'a>) -> Self {
        Self {
            languages,
            document_lines,
            line_index: 0,
        }
    }
}

impl<'a> Iterator for MarkdownIterator<'a> {
    type Item = MarkdownToken;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.document_lines.next() {
            self.line_index += 1;

            // found the start of a code block (=testcase)?
            if let Some((backticks, language)) = extract_code_block_start(line) {
                if language.is_empty() || !self.languages.contains(&language) {
                    return Some(MarkdownToken::Line(self.line_index - 1, line.into()));
                }

                // gather comments before command
                let mut line = self.document_lines.next()?;
                self.line_index += 1;
                let mut comment_lines = vec![];
                while is_comment(line) {
                    comment_lines.push((self.line_index - 1, line.to_string()));
                    line = self.document_lines.next()?;
                    self.line_index += 1;
                }

                // gather code until then end
                let mut code_lines = vec![];
                while !line.starts_with(backticks) {
                    code_lines.push((self.line_index - 1, line.to_string()));
                    line = self.document_lines.next()?;
                    self.line_index += 1;
                }

                Some(MarkdownToken::CodeBlock {
                    code_lines,
                    comment_lines,
                    language: language.into(),
                })

            // not a code block -> just gather the line
            } else {
                Some(MarkdownToken::Line(self.line_index - 1, line.into()))
            }
        } else {
            None
        }
    }
}

fn extract_header(line: &str) -> Option<(String, String)> {
    HEADER_LINE.captures(line).map(|captures| {
        (
            captures.get(1).unwrap().as_str().to_string(),
            captures.get(2).unwrap().as_str().to_string(),
        )
    })
}

/// Parses a markdown document line and returns the content of that line if it
/// is either a paragraph or a header (without the prefixed `#`)
pub(crate) fn extract_title(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if PARAGRAPH_START.is_match(line) {
        Some(("".into(), line.into()))
    } else {
        extract_header(line)
    }
}

/// Parses a markdown code block starting line of three (or more) backticks
/// that may be followed by a language.
///
/// For example:
///
/// ````markdown
/// ```foo
/// code block
/// ```
/// ````
///
/// On the first line ending in foo, this function returns the backticks and
/// the language. On all other lines it returns None.
pub(crate) fn extract_code_block_start(line: &str) -> Option<(&str, &str)> {
    for (index, ch) in line.chars().enumerate() {
        if ch != '`' {
            if index < 2 {
                return None;
            }
            return Some((&line[0..index], &line[index..]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::MarkdownParser;
    use crate::expectation::tests::expectation_maker;
    use crate::parsers::markdown::DEFAULT_MARKDOWN_LANGUAGES;
    use crate::parsers::parser::Parser;
    use crate::test_expectation;
    use crate::testcase::TestCase;

    fn parser() -> MarkdownParser {
        let maker = expectation_maker();
        MarkdownParser::new(Arc::new(maker), DEFAULT_MARKDOWN_LANGUAGES)
    }

    #[test]
    fn test_markdown_simple() {
        let cram_test = r#"
This is a title

```scrut
$ echo hello
hello
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 5,
                ..Default::default()
            },
            testcases[0]
        );
    }

    #[test]
    fn test_title_from_nearest_line() {
        let cram_test = r#"
Something here

Something there

This is a title

```scrut
$ echo hello
hello
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 9,
                ..Default::default()
            },
            testcases[0]
        );
    }

    #[test]
    fn test_title_from_full_paragraph() {
        let cram_test = r#"
Not a title

This is a title
This is still part of it
And another part of the title

```scrut
$ echo hello
hello
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title\nThis is still part of it\nAnd another part of the title"
                    .to_string(),
                exit_code: None,
                line_number: 9,
                ..Default::default()
            },
            testcases[0]
        );
    }

    #[test]
    fn test_title_from_header() {
        let cram_test = r#"
Something

### This is a title

```scrut
$ echo hello
hello
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 7,
                ..Default::default()
            },
            testcases[0]
        );
    }

    #[test]
    fn test_comment_before_command_is_ignored() {
        let cram_test = r#"
# This is a title

```scrut
# ignore
# me
$ echo hello
hello
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 7,
                ..Default::default()
            },
            testcases[0]
        );
    }

    #[test]
    fn test_code_only_from_specified_languages() {
        let cram_test = r#"
This is a title1

```text
$ echo hello1
hello1
```

This is a title

```scrut
$ echo hello
hello
```
This is a title3

```bla
$ echo hello3
hello3
```


This is another title

```testcase
$ echo world
world
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(2, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "echo hello".to_string(),
                expectations: vec![test_expectation!("equal", "hello", false, false)],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 12,
                ..Default::default()
            },
            testcases[0]
        );
        assert_eq!(
            TestCase {
                shell_expression: "echo world".to_string(),
                expectations: vec![test_expectation!("equal", "world", false, false)],
                title: "This is another title".to_string(),
                exit_code: None,
                line_number: 26,
                ..Default::default()
            },
            testcases[1]
        );
    }

    #[test]
    fn test_commands_only_composed_of_initial_elements() {
        let cram_test = r#"
Something

### This is a title

```scrut
$ i am command 1
> i am command 2
i am output 1
> i am output 2
i am output 3
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            TestCase {
                shell_expression: "i am command 1\ni am command 2".into(),
                expectations: vec![
                    test_expectation!("equal", "i am output 1", false, false),
                    test_expectation!("equal", "> i am output 2", false, false),
                    test_expectation!("equal", "i am output 3", false, false),
                ],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 7,
                ..Default::default()
            },
            testcases[0]
        );
    }

    #[test]
    fn test_markdown_with_extended_code_block() {
        let cram_test = r#"
This is a title

````scrut
$ echo hello
```scrut
inner
```
text
````

And another title

````scrut
$ cat test.md
# Command executes successfully

```scrut
$ echo Hello World
Hello World
```
````
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(2, testcases.len());
        assert_eq!(
            vec![
                TestCase {
                    shell_expression: "echo hello".to_string(),
                    expectations: vec![
                        test_expectation!("equal", "```scrut"),
                        test_expectation!("equal", "inner"),
                        test_expectation!("equal", "```"),
                        test_expectation!("equal", "text"),
                    ],
                    title: "This is a title".to_string(),
                    exit_code: None,
                    line_number: 5,
                    ..Default::default()
                },
                TestCase {
                    shell_expression: "cat test.md".to_string(),
                    expectations: vec![
                        test_expectation!("equal", "# Command executes successfully"),
                        test_expectation!("equal", ""),
                        test_expectation!("equal", "```scrut"),
                        test_expectation!("equal", "$ echo Hello World"),
                        test_expectation!("equal", "Hello World"),
                        test_expectation!("equal", "```"),
                    ],
                    title: "And another title".to_string(),
                    exit_code: None,
                    line_number: 15,
                    ..Default::default()
                },
            ],
            testcases
        );
    }

    #[test]
    fn test_output_of_dollar_lines() {
        let cram_test = r#"
This is a title

```scrut
$ echo -e '$ hello\nworld'
$ hello
world
```
"#;
        let parser = parser();
        let testcases = parser.parse(cram_test).expect("must parse");
        assert_eq!(1, testcases.len());
        assert_eq!(
            vec![TestCase {
                shell_expression: "echo -e '$ hello\\nworld'".to_string(),
                expectations: vec![
                    test_expectation!("equal", "$ hello"),
                    test_expectation!("equal", "world"),
                ],
                title: "This is a title".to_string(),
                exit_code: None,
                line_number: 5,
                ..Default::default()
            },],
            testcases
        );
    }
}
