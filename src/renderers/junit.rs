/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use chrono::DateTime;
use chrono::Local;
use quick_xml::Writer;
use quick_xml::events::BytesDecl;
use quick_xml::events::BytesText;
use quick_xml::events::Event;

use super::pretty::PrettyColorRenderer;
use super::pretty::PrettyMonochromeRenderer;
use super::renderer::Renderer;
use crate::escaping::Escaper;
use crate::newline::BytesNewline;
use crate::newline::SplitLinesByNewline;
use crate::outcome::Outcome;
use crate::testcase::TestCaseError;
use crate::validation::JsonSchemaFailureKind;
use crate::validation::ValidationFailure;

/// Suite name for outcomes that do not know where they came from
const UNKNOWN_LOCATION: &str = "<unknown>";

/// Renders outcomes as JUnit XML, the report format that CI test result
/// collectors (GitHub Actions, Jenkins, GitLab, ..) consume.
#[derive(Default)]
pub struct JunitRenderer {
    base_directory: Option<PathBuf>,
    timestamp: Option<DateTime<Local>>,
}

impl JunitRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stamp every `<testsuite>` with `at`. One time is used for all of them:
    /// Scrut records how long each test case took but not when it began, so
    /// there is no per-document start time to report.
    pub fn with_timestamp(mut self, at: DateTime<Local>) -> Self {
        self.timestamp = Some(at);
        self
    }

    /// [`Self::with_timestamp`] reading the clock, so that callers do not need
    /// a dependency on `chrono` of their own. Tests pass a fixed time instead.
    ///
    /// The clock is read here, when the renderer is built, not when the report
    /// is written. The CLI builds the renderer once the whole run is over, so
    /// the stamp marks the end of the run.
    pub fn with_current_timestamp(self) -> Self {
        let now = Local::now();
        self.with_timestamp(now)
    }

    /// Report document paths relative to `directory`. Annotation tooling maps
    /// the `file` attribute onto files in the repository, which only works for
    /// paths relative to the checkout -- but Scrut reports a document wherever
    /// the invocation pointed at it, which may be absolute or reach out of the
    /// working directory.
    pub fn with_base_directory(mut self, directory: PathBuf) -> Self {
        self.base_directory = Some(directory);
        self
    }
}

impl Renderer for JunitRenderer {
    fn render(&self, outcomes: &[&Outcome]) -> Result<String> {
        let suites = to_suites(outcomes, self.base_directory.as_deref(), self.timestamp)?;
        let totals = Counts::of(suites.iter().flat_map(|suite| &suite.testcases));

        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
        writer
            .create_element("testsuites")
            .with_attributes(attributes(&totals.attributes()))
            .write_inner_content(|writer| {
                suites
                    .iter()
                    .try_for_each(|suite| write_suite(writer, suite))
            })?;
        writer.write_event(Event::Text(BytesText::from_escaped("\n")))?;

        Ok(String::from_utf8(writer.into_inner())?)
    }
}

/// A `<testsuite>`: all testcases of one test document
struct Suite {
    name: String,
    /// Directory the document lives in, the closest analogue Scrut has to the
    /// Java package the attribute was designed for. Report viewers group by it.
    package: Option<String>,
    properties: Vec<(String, String)>,
    timestamp: Option<DateTime<Local>>,
    testcases: Vec<Case>,
}

/// A `<testcase>`, fully resolved so that serialization itself cannot fail
struct Case {
    name: String,
    classname: String,
    line_number: usize,
    duration: Option<Duration>,
    status: Status,
    system_out: String,
    system_err: String,
}

/// What became of a testcase. `kind` is the same vocabulary the JSON renderer
/// uses, `detail` is the human readable explanation of the failure.
enum Status {
    Success,
    Skipped {
        message: &'static str,
    },
    Failure {
        kind: &'static str,
        message: String,
        detail: String,
    },
    Error {
        kind: &'static str,
        message: String,
        detail: String,
    },
}

/// Tally of testcase states, rendered as attributes on both `<testsuite>` and
/// the root `<testsuites>`
#[derive(Default)]
struct Counts {
    tests: usize,
    failures: usize,
    errors: usize,
    skipped: usize,
    time: Duration,
}

impl Counts {
    fn of<'a>(cases: impl IntoIterator<Item = &'a Case>) -> Self {
        let mut counts = Self::default();
        for case in cases {
            counts.tests += 1;
            counts.time += case.duration.unwrap_or_default();
            match case.status {
                Status::Success => {}
                Status::Skipped { .. } => counts.skipped += 1,
                Status::Failure { .. } => counts.failures += 1,
                Status::Error { .. } => counts.errors += 1,
            }
        }
        counts
    }

    fn attributes(&self) -> Vec<(&'static str, String)> {
        vec![
            ("tests", self.tests.to_string()),
            ("failures", self.failures.to_string()),
            ("errors", self.errors.to_string()),
            ("skipped", self.skipped.to_string()),
            ("time", seconds(self.time)),
        ]
    }
}

/// Groups outcomes into one suite per test document, preserving the order in
/// which the documents were executed
fn to_suites(
    outcomes: &[&Outcome],
    base_directory: Option<&Path>,
    timestamp: Option<DateTime<Local>>,
) -> Result<Vec<Suite>> {
    let mut suites: Vec<Suite> = vec![];
    for outcome in outcomes {
        let location = match outcome.location {
            Some(ref location) => report_path(location, base_directory),
            None => UNKNOWN_LOCATION.to_string(),
        };
        let name = attribute_text(&location, &outcome.escaping);
        let case = to_case(outcome, &name)?;
        match suites.iter_mut().find(|suite| suite.name == name) {
            Some(suite) => suite.testcases.push(case),
            None => suites.push(Suite {
                package: package_of(&name),
                name,
                properties: vec![("scrut.format".to_string(), outcome.format.to_string())],
                timestamp,
                testcases: vec![case],
            }),
        }
    }

    suites.iter_mut().for_each(deduplicate_names);
    Ok(suites)
}

fn to_case(outcome: &Outcome, classname: &str) -> Result<Case> {
    let name = if outcome.testcase.title.is_empty() {
        format!("line {}", outcome.testcase.line_number)
    } else {
        attribute_text(&outcome.testcase.title, &outcome.escaping)
    };

    Ok(Case {
        name,
        classname: classname.to_string(),
        line_number: outcome.testcase.line_number,
        duration: outcome.output.duration,
        status: to_status(outcome)?,
        system_out: printable((&outcome.output.stdout).into(), &outcome.escaping),
        system_err: printable((&outcome.output.stderr).into(), &outcome.escaping),
    })
}

fn to_status(outcome: &Outcome) -> Result<Status> {
    let Err(ref err) = outcome.result else {
        return Ok(Status::Success);
    };

    let (kind, message) = match err {
        TestCaseError::ValidationFailed(ValidationFailure::MalformedOutput(_)) => (
            "malformed_output",
            "output does not match expectations".to_string(),
        ),
        TestCaseError::ValidationFailed(ValidationFailure::JsonSchemaFailed(failure)) => {
            match failure.kind {
                JsonSchemaFailureKind::InvalidSchema => (
                    "json_schema_invalid_schema",
                    "JSON Schema definition is invalid".to_string(),
                ),
                JsonSchemaFailureKind::InvalidJson => (
                    "json_schema_invalid_json",
                    "output is not valid JSON".to_string(),
                ),
                JsonSchemaFailureKind::ValidationErrors => (
                    "json_schema_validation_errors",
                    "output does not conform to JSON Schema".to_string(),
                ),
            }
        }
        TestCaseError::InvalidExitCode { actual, expected } => (
            "invalid_exit_code",
            format!("unexpected exit code: expected {expected}, but got {actual}"),
        ),
        TestCaseError::Timeout => ("timeout", "execution timed out".to_string()),
        TestCaseError::InternalError(err) => ("internal_error", err.to_string()),
        // Scrut skips a test case either because the document opted out via the
        // skip exit code, or because an earlier case timed out or tripped
        // `fail_fast`. `TestCaseError::Skipped` does not say which, so the
        // message states only what is true of both.
        TestCaseError::Skipped => {
            return Ok(Status::Skipped {
                message: "test case was not executed",
            });
        }
    };

    // reuse the renderer that engineers already read in the terminal, so the
    // report shows the very same diff. It echoes the title and shell expression
    // back verbatim, so it needs the same escaping as any other emitted text.
    let detail = PrettyMonochromeRenderer::new(PrettyColorRenderer {
        summarize: false,
        ..Default::default()
    })
    .render(&[outcome])?;
    let detail = printable(detail.trim_end().as_bytes(), &outcome.escaping);
    let message = attribute_text(&message, &outcome.escaping);

    Ok(match err {
        TestCaseError::Timeout | TestCaseError::InternalError(_) => Status::Error {
            kind,
            message,
            detail,
        },
        _ => Status::Failure {
            kind,
            message,
            detail,
        },
    })
}

/// JUnit consumers key on `classname` + `name`, but Scrut titles need not be
/// unique within a document, so collisions get a numeric suffix.
fn deduplicate_names(suite: &mut Suite) {
    let mut taken = HashSet::new();
    for case in &mut suite.testcases {
        if taken.insert(case.name.clone()) {
            continue;
        }
        let mut counter = 2;
        let name = loop {
            let candidate = format!("{} #{}", case.name, counter);
            if taken.insert(candidate.clone()) {
                break candidate;
            }
            counter += 1;
        };
        case.name = name;
    }
}

/// Renders a document location for the report: relative to the base directory
/// when it sits underneath it, and always with `/` separators, since that is
/// what consumers expect regardless of the platform the tests ran on. A
/// location that does not sit under the base directory is reported unchanged --
/// a wrong relative path would be worse than an honest absolute one.
///
/// The comparison is lexical, on path components, so `.` segments and trailing
/// separators do not throw it off. It does not touch the filesystem, which
/// leaves one case unhandled: a document reached through a different symlink
/// than the one the working directory was resolved to shares no prefix and so
/// keeps its absolute path. Resolving that would mean canonicalizing inside the
/// renderer, which is filesystem IO that can fail, that would make rendering
/// non-deterministic, and that on Windows yields `\\?\` paths no consumer wants.
fn report_path(location: &str, base_directory: Option<&Path>) -> String {
    let path = Path::new(location);
    let relative = base_directory
        .and_then(|base| path.strip_prefix(base).ok())
        .unwrap_or(path);
    let rendered = relative.display().to_string();
    if cfg!(windows) {
        rendered.replace('\\', "/")
    } else {
        // on unix a backslash is a legal filename character, not a separator
        rendered
    }
}

/// The directory part of an already report-formatted location, or `None` for a
/// document that sits at the root of the run and so has nothing to group under.
fn package_of(location: &str) -> Option<String> {
    location
        .rsplit_once('/')
        .map(|(directory, _)| directory.to_string())
        .filter(|directory| !directory.is_empty())
}

/// Renders text so that it is valid XML character data. XML 1.0 forbids most
/// control characters outright -- escaping them as entities does not make them
/// legal -- and captured output need not even be valid UTF-8. Scrut's escaper
/// replaces both with printable escape sequences; applying it line by line
/// keeps the line structure of multi-line content intact.
fn printable(text: &[u8], escaping: &Escaper) -> String {
    let escaped = text
        .split_at_newline()
        .iter()
        .map(|line| escaping.escaped_printable(line.trim_newlines()))
        .collect::<Vec<_>>()
        .join("\n");
    xml_safe(escaped)
}

/// The characters XML 1.0 permits in a document. Note this is narrower than
/// what [`Escaper`] escapes: the escaper exists to make output readable in a
/// terminal, so it leaves alone code points that are printable but illegal in
/// XML, such as the noncharacter `U+FFFF`.
fn is_legal_xml_char(c: char) -> bool {
    matches!(c,
        '\t' | '\n' | '\r'
        | '\u{20}'..='\u{d7ff}'
        | '\u{e000}'..='\u{fffd}'
        | '\u{10000}'..='\u{10ffff}')
}

/// Backstop behind [`Escaper`] that hex-escapes anything still illegal in XML,
/// in the same notation the escaper itself uses. Without it such a character
/// would make the whole report unparseable -- entity-encoding does not help,
/// XML forbids the character itself.
fn xml_safe(text: String) -> String {
    if text.chars().all(is_legal_xml_char) {
        return text;
    }
    let mut buffer = [0; 4];
    text.chars()
        .map(|c| {
            if is_legal_xml_char(c) {
                c.to_string()
            } else {
                Escaper::Ascii.escaped_printable(c.encode_utf8(&mut buffer).as_bytes())
            }
        })
        .collect()
}

/// Same, for attribute values. Newlines are legal in an attribute but every
/// parser normalizes them to spaces on read, so collapse them here and keep the
/// value round-trippable.
fn attribute_text(text: &str, escaping: &Escaper) -> String {
    printable(text.as_bytes(), escaping).replace('\n', " ")
}

fn seconds(duration: Duration) -> String {
    format!("{:.3}", duration.as_secs_f64())
}

fn attributes<'a>(pairs: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    pairs.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

fn write_suite<W: Write>(writer: &mut Writer<W>, suite: &Suite) -> io::Result<()> {
    let mut attrs = vec![("name", suite.name.clone())];
    if let Some(ref package) = suite.package {
        attrs.push(("package", package.clone()));
    }
    if let Some(timestamp) = suite.timestamp {
        // local time without an offset, as the Ant schema's `timestamp` pattern
        // requires -- appending one makes the report fail strict validation
        attrs.push((
            "timestamp",
            timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
        ));
    }
    attrs.extend(Counts::of(&suite.testcases).attributes());

    writer
        .create_element("testsuite")
        .with_attributes(attributes(&attrs))
        .write_inner_content(|writer| {
            write_properties(writer, &suite.properties)?;
            suite
                .testcases
                .iter()
                .try_for_each(|case| write_case(writer, case))
        })?;
    Ok(())
}

fn write_properties<W: Write>(
    writer: &mut Writer<W>,
    properties: &[(String, String)],
) -> io::Result<()> {
    if properties.is_empty() {
        return Ok(());
    }
    writer
        .create_element("properties")
        .write_inner_content(|writer| {
            properties.iter().try_for_each(|(key, value)| {
                writer
                    .create_element("property")
                    .with_attributes([("name", key.as_str()), ("value", value.as_str())])
                    .write_empty()
                    .map(|_| ())
            })
        })?;
    Ok(())
}

fn write_case<W: Write>(writer: &mut Writer<W>, case: &Case) -> io::Result<()> {
    let mut attrs = vec![
        ("name", case.name.clone()),
        ("classname", case.classname.clone()),
        ("file", case.classname.clone()),
        ("line", case.line_number.to_string()),
    ];
    if let Some(duration) = case.duration {
        attrs.push(("time", seconds(duration)));
    }

    let element = writer
        .create_element("testcase")
        .with_attributes(attributes(&attrs));

    let has_body = !matches!(case.status, Status::Success)
        || !case.system_out.is_empty()
        || !case.system_err.is_empty();
    if !has_body {
        element.write_empty()?;
        return Ok(());
    }

    element.write_inner_content(|writer| {
        match &case.status {
            Status::Success => {}
            Status::Skipped { message } => {
                writer
                    .create_element("skipped")
                    .with_attribute(("message", *message))
                    .write_empty()?;
            }
            Status::Failure {
                kind,
                message,
                detail,
            } => write_problem(writer, "failure", kind, message, detail)?,
            Status::Error {
                kind,
                message,
                detail,
            } => write_problem(writer, "error", kind, message, detail)?,
        }
        write_stream(writer, "system-out", &case.system_out)?;
        write_stream(writer, "system-err", &case.system_err)
    })?;
    Ok(())
}

fn write_problem<W: Write>(
    writer: &mut Writer<W>,
    element: &str,
    kind: &str,
    message: &str,
    detail: &str,
) -> io::Result<()> {
    writer
        .create_element(element)
        .with_attributes([("type", kind), ("message", message)])
        .write_text_content(BytesText::new(detail))?;
    Ok(())
}

fn write_stream<W: Write>(writer: &mut Writer<W>, element: &str, content: &str) -> io::Result<()> {
    if content.is_empty() {
        return Ok(());
    }
    writer
        .create_element(element)
        .write_text_content(BytesText::new(content))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::Duration;

    use anyhow::Result;
    use anyhow::anyhow;
    use chrono::Local;
    use chrono::TimeZone;
    use quick_xml::Reader;
    use quick_xml::XmlVersion;
    use quick_xml::events::BytesStart;
    use quick_xml::events::Event;

    use super::JunitRenderer;
    use super::is_legal_xml_char;
    use crate::escaping::Escaper;
    use crate::outcome::Outcome;
    use crate::output::ExitStatus;
    use crate::output::Output;
    use crate::parsers::parser::ParserType;
    use crate::renderers::renderer::Renderer;
    use crate::test_expectation;
    use crate::testcase::TestCase;
    use crate::testcase::TestCaseError;
    use crate::validation::OutputBody;
    use crate::validation::ValidationBody;

    /// What a rendered report contains, recovered by parsing it back
    #[derive(Default)]
    struct Report {
        root: BTreeMap<String, String>,
        suites: usize,
        cases: usize,
        failures: usize,
        errors: usize,
        skipped: usize,
        case_names: Vec<String>,
        system_out: Vec<String>,
    }

    fn element_attributes(element: &BytesStart) -> BTreeMap<String, String> {
        element
            .attributes()
            .map(|attribute| {
                let attribute = attribute.expect("attribute parses");
                (
                    attribute.key.as_ref().to_string(),
                    attribute
                        .normalized_value(XmlVersion::Explicit1_0)
                        .expect("attribute value normalizes")
                        .to_string(),
                )
            })
            .collect()
    }

    fn record(report: &mut Report, name: &str, attributes: BTreeMap<String, String>) {
        match name {
            "testsuites" => report.root = attributes,
            "testsuite" => report.suites += 1,
            "testcase" => {
                report.cases += 1;
                report
                    .case_names
                    .push(attributes.get("name").cloned().unwrap_or_default());
            }
            "failure" => report.failures += 1,
            "error" => report.errors += 1,
            "skipped" => report.skipped += 1,
            _ => {}
        }
    }

    /// Parses the rendered report back, asserting that it is well-formed XML
    /// built only from legal characters, and that the counts it declares match
    /// the elements actually present. `quick-xml` is lenient about illegal
    /// characters, so that has to be checked separately rather than relying on
    /// the parse to reject them.
    fn parse_report(xml: &str) -> Report {
        if let Some((at, c)) = xml.char_indices().find(|(_, c)| !is_legal_xml_char(*c)) {
            panic!(
                "illegal XML character {c:?} (U+{:04X}) at byte {at} of:\n{xml}",
                c as u32
            );
        }

        let mut reader = Reader::from_str(xml);
        let mut report = Report::default();
        let mut in_system_out = false;
        // the reader splits character data at entity references, so text has to
        // be reassembled across events to check it round-trips
        let mut system_out = String::new();
        loop {
            let event = reader
                .read_event()
                .unwrap_or_else(|err| panic!("report is not well-formed XML: {err}\n{xml}"));
            match event {
                Event::Eof => break,
                Event::Start(element) => {
                    let name = element.name().as_ref().to_string();
                    record(&mut report, &name, element_attributes(&element));
                    in_system_out = name == "system-out";
                }
                Event::Empty(element) => {
                    let name = element.name().as_ref().to_string();
                    record(&mut report, &name, element_attributes(&element));
                }
                Event::Text(text) if in_system_out => system_out.push_str(&text),
                Event::GeneralRef(entity) if in_system_out => {
                    let resolved = entity
                        .resolve_char_ref()
                        .expect("character reference resolves")
                        .or_else(|| match entity.as_ref() {
                            "lt" => Some('<'),
                            "gt" => Some('>'),
                            "amp" => Some('&'),
                            "apos" => Some('\''),
                            "quot" => Some('"'),
                            _ => None,
                        })
                        .unwrap_or_else(|| panic!("known entity `&{};`", entity.as_ref()));
                    system_out.push(resolved);
                }
                Event::End(_) => {
                    if in_system_out {
                        report.system_out.push(std::mem::take(&mut system_out));
                    }
                    in_system_out = false;
                }
                _ => {}
            }
        }

        let declared = |key: &str| -> usize {
            report
                .root
                .get(key)
                .unwrap_or_else(|| panic!("root declares `{key}`"))
                .parse()
                .expect("count is a number")
        };
        assert_eq!(
            declared("tests"),
            report.cases,
            "declared `tests` matches the number of testcase elements"
        );
        assert_eq!(
            declared("failures"),
            report.failures,
            "declared `failures` matches the number of failure elements"
        );
        assert_eq!(
            declared("errors"),
            report.errors,
            "declared `errors` matches the number of error elements"
        );
        assert_eq!(
            declared("skipped"),
            report.skipped,
            "declared `skipped` matches the number of skipped elements"
        );

        report
    }

    /// Builds an outcome with a fixed duration, so that rendered output is
    /// deterministic and can be snapshot tested
    fn outcome(
        location: Option<&str>,
        title: &str,
        line_number: usize,
        output: Output,
        result: Result<(), TestCaseError>,
    ) -> Outcome {
        Outcome {
            location: location.map(str::to_string),
            output,
            testcase: TestCase {
                title: title.to_string(),
                shell_expression: "the command".to_string(),
                body: ValidationBody::Output(OutputBody {
                    expectations: vec![test_expectation!("equal", "expected")],
                }),
                exit_code: Some(0),
                line_number,
                ..Default::default()
            },
            escaping: Escaper::default(),
            format: ParserType::Markdown,
            result,
        }
    }

    fn timed(output: impl Into<Output>, millis: u64) -> Output {
        Output {
            duration: Some(Duration::from_millis(millis)),
            ..output.into()
        }
    }

    /// Renders and, for every case, asserts the result is actually parseable
    /// XML rather than merely the bytes a snapshot happens to pin
    fn render(outcomes: &[Outcome]) -> Result<String> {
        let rendered = JunitRenderer::new().render(&outcomes.iter().collect::<Vec<_>>())?;
        parse_report(&rendered);
        Ok(rendered)
    }

    #[test]
    fn test_success_renders_empty_testcase_with_time() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 1500),
            Ok(()),
        )])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_invalid_exit_code_renders_failure() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            timed(("the stdout\n", "the stderr\n", Some(123)), 250),
            Err(TestCaseError::InvalidExitCode {
                actual: 123,
                expected: 0,
            }),
        )])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_timeout_renders_error() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            timed(("", "", None), 900),
            Err(TestCaseError::Timeout),
        )])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_internal_error_renders_error() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            timed(("", "", None), 10),
            Err(TestCaseError::InternalError(anyhow!("something broke"))),
        )])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_skipped_renders_skipped() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            ("", "", None).into(),
            Err(TestCaseError::Skipped),
        )])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_multiple_documents_render_multiple_suites() {
        let rendered = render(&[
            outcome(
                Some("path/one.md"),
                "first",
                3,
                timed(("out 1\n", "", Some(0)), 100),
                Ok(()),
            ),
            outcome(
                Some("path/two.md"),
                "second",
                4,
                timed(("out 2\n", "", Some(0)), 200),
                Ok(()),
            ),
            outcome(
                Some("path/one.md"),
                "third",
                9,
                timed(("out 3\n", "", Some(0)), 300),
                Ok(()),
            ),
        ])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_duplicate_titles_are_suffixed() {
        let rendered = render(&[
            outcome(
                Some("path/file.md"),
                "same title",
                3,
                timed(("", "", Some(0)), 10),
                Ok(()),
            ),
            outcome(
                Some("path/file.md"),
                "same title",
                7,
                timed(("", "", Some(0)), 10),
                Ok(()),
            ),
            outcome(
                Some("path/file.md"),
                "same title",
                11,
                timed(("", "", Some(0)), 10),
                Ok(()),
            ),
        ])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_non_printable_output_is_escaped() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            timed(("before\u{7}after\nsecond\u{0}line\n", "", Some(0)), 10),
            Ok(()),
        )])
        .expect("rendering succeeds");
        assert!(
            !rendered.contains('\u{7}') && !rendered.contains('\u{0}'),
            "no raw control characters survive into the XML: {rendered}"
        );
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_missing_duration_omits_time_attribute() {
        let rendered = render(&[outcome(
            Some("path/file.md"),
            "the title",
            12,
            ("", "", Some(0)).into(),
            Ok(()),
        )])
        .expect("rendering succeeds");
        let testcase = rendered
            .lines()
            .find(|line| line.trim_start().starts_with("<testcase"))
            .expect("report contains a testcase element");
        assert!(
            !testcase.contains("time="),
            "testcase without a duration carries no time attribute: {testcase}"
        );
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_report_path_strips_lexically() {
        let tests = [
            (
                "/repo/checkout",
                "/repo/checkout/tests/file.md",
                "tests/file.md",
                "a path under the base directory is made relative",
            ),
            (
                "/repo/checkout/",
                "/repo/checkout/tests/file.md",
                "tests/file.md",
                "a trailing separator on the base directory makes no difference",
            ),
            (
                "/repo/checkout",
                "/repo/checkout/./tests/file.md",
                "tests/file.md",
                "a `.` segment makes no difference",
            ),
            (
                "/repo/checkout",
                "/repo/checkout/tests/../tests/file.md",
                "tests/../tests/file.md",
                "a `..` segment is kept, since dropping it would change which file the path names",
            ),
            (
                "/repo/checkout",
                "tests/file.md",
                "tests/file.md",
                "an already relative path is left alone",
            ),
            (
                "/repo/checkout",
                "/elsewhere/file.md",
                "/elsewhere/file.md",
                "a path outside the base directory keeps its absolute form",
            ),
        ];

        for (base, location, expected, reason) in tests {
            assert_eq!(
                expected,
                &super::report_path(location, Some(Path::new(base))),
                "{reason}"
            );
        }
    }

    #[test]
    fn test_suite_carries_timestamp_when_given() {
        let at = Local
            .with_ymd_and_hms(2026, 9, 17, 13, 45, 6)
            .single()
            .expect("unambiguous local time");
        let outcomes = [outcome(
            Some("file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )];
        let rendered = JunitRenderer::new()
            .with_timestamp(at)
            .render(&outcomes.iter().collect::<Vec<_>>())
            .expect("rendering succeeds");
        parse_report(&rendered);

        assert!(
            rendered.contains(r#"timestamp="2026-09-17T13:45:06""#),
            "timestamp is local time with no offset, per the Ant schema: {rendered}"
        );
    }

    #[test]
    fn test_suite_has_no_timestamp_by_default() {
        let rendered = render(&[outcome(
            Some("file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )])
        .expect("rendering succeeds");
        assert!(
            !rendered.contains("timestamp="),
            "no timestamp is invented when the caller did not supply one: {rendered}"
        );
    }

    #[test]
    fn test_suite_carries_package_and_properties() {
        let rendered = render(&[outcome(
            Some("tests/cases/file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )])
        .expect("rendering succeeds");
        assert!(
            rendered.contains(r#"package="tests/cases""#),
            "suite is grouped by the directory the document lives in: {rendered}"
        );
        assert!(
            rendered.contains(r#"<property name="scrut.format" value="markdown"/>"#),
            "suite records the format the document was written in: {rendered}"
        );
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_document_at_the_root_has_no_package() {
        let rendered = render(&[outcome(
            Some("file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )])
        .expect("rendering succeeds");
        assert!(
            !rendered.contains("package="),
            "a document with no directory is not given an empty package: {rendered}"
        );
    }

    #[test]
    fn test_paths_are_reported_relative_to_the_base_directory() {
        let outcomes = [outcome(
            Some("/repo/checkout/tests/cases/file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )];
        let rendered = JunitRenderer::new()
            .with_base_directory(PathBuf::from("/repo/checkout"))
            .render(&outcomes.iter().collect::<Vec<_>>())
            .expect("rendering succeeds");
        parse_report(&rendered);

        assert!(
            rendered.contains(r#"file="tests/cases/file.md""#),
            "path is reported relative to the base directory: {rendered}"
        );
        assert!(
            !rendered.contains("/repo/checkout"),
            "no absolute path leaks into the report: {rendered}"
        );
    }

    #[test]
    fn test_paths_outside_the_base_directory_are_left_alone() {
        let outcomes = [outcome(
            Some("/elsewhere/file.md"),
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )];
        let rendered = JunitRenderer::new()
            .with_base_directory(PathBuf::from("/repo/checkout"))
            .render(&outcomes.iter().collect::<Vec<_>>())
            .expect("rendering succeeds");
        parse_report(&rendered);

        assert!(
            rendered.contains(r#"file="/elsewhere/file.md""#),
            "a path that is not under the base directory is reported unchanged: {rendered}"
        );
    }

    #[test]
    fn test_hostile_content_stays_well_formed_xml() {
        // everything a test document and its output can throw at an XML writer:
        // markup metacharacters in attribute positions, a CDATA terminator,
        // control characters, invalid UTF-8 and a non-character code point
        let mut stdout = b"plain line\n".to_vec();
        stdout.extend_from_slice(b"markup < > & \" ' and a ]]> terminator\n");
        stdout.extend_from_slice(&[0x00, 0x07, 0x1b, b'\n']);
        stdout.extend_from_slice(&[0xff, 0xfe, b'\n']);
        stdout.extend_from_slice("emoji \u{1f980} and noncharacter \u{ffff}\n".as_bytes());

        let mut outcome = outcome(
            Some("path/<dir> & \"quoted\"/file.md"),
            "a \"quoted\" <title> & a bell \u{7} and an escape \u{1b}",
            12,
            Output {
                stdout: stdout.into(),
                stderr: vec![].into(),
                exit_code: ExitStatus::Code(0),
                detached_process: None,
                captured_env: BTreeMap::new(),
                duration: Some(Duration::from_millis(5)),
            },
            Ok(()),
        );
        outcome.testcase.shell_expression = "echo \u{1b}[31mred\u{1b}[0m && exit <0>".to_string();

        let rendered = render(&[outcome]).expect("rendering succeeds");
        let report = parse_report(&rendered);

        // the markup characters survive as data rather than becoming markup ..
        assert_eq!(
            report.case_names,
            vec!["a \"quoted\" <title> & a bell \\a and an escape \\x1b".to_string()],
            "title round-trips with only the control characters escaped"
        );
        // .. and the control characters never reach the document verbatim
        assert!(
            !rendered.contains('\u{7}') && !rendered.contains('\u{1b}'),
            "no raw control characters survive into the XML: {rendered}"
        );
        assert!(
            report
                .system_out
                .iter()
                .any(|out| out.contains("]]>") && out.contains("\\x00")),
            "output round-trips with markup as data and control bytes escaped: {:?}",
            report.system_out
        );

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_empty_run_renders_empty_report() {
        let rendered = render(&[]).expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_missing_location_uses_placeholder_suite() {
        let rendered = render(&[outcome(
            None,
            "the title",
            12,
            timed(("", "", Some(0)), 10),
            Ok(()),
        )])
        .expect("rendering succeeds");
        insta::assert_snapshot!(rendered);
    }
}
