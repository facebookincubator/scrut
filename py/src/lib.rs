use std::collections::HashMap;
use std::convert::From;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use lazy_static::lazy_static;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use regex::Regex;
use scrut::config::DocumentConfig;
use scrut::config::OutputStreamControl;
use scrut::config::TestCaseConfig;
use scrut::config::TestCaseWait;
use scrut::escaping::Escaper;
use scrut::expectation::ExpectationMaker;
use scrut::outcome::Outcome;
use scrut::output::ExitStatus;
use scrut::output::Output;
use scrut::parsers::cram::CramParser;
use scrut::parsers::markdown::MarkdownParser;
use scrut::parsers::parser::Parser;
use scrut::parsers::parser::ParserType;
use scrut::renderers::pretty::PrettyMonochromeRenderer;
use scrut::renderers::renderer::Renderer;
use scrut::rules::glob_cram::CramGlobRule;
use scrut::rules::registry::RuleRegistry;
use scrut::rules::rule::RuleMaker;
use scrut::testcase::TestCase;

lazy_static! {
    static ref EXIT_CODE: Regex = Regex::new("^(\\d+)").expect("exit code regex");
    static ref EXIT_TIMEOUT: Regex =
        Regex::new("^timeout\\[(\\d+)ms\\]$").expect("exit timeout regex");
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyscrut(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyOutput>()?;
    m.add_class::<PyTestCase>()?;
    m.add_class::<PyTestCaseWait>()?;
    m.add_class::<PyTestCaseConfig>()?;
    m.add_class::<PyDocumentConfig>()?;
    m.add_class::<PyCramParser>()?;
    m.add_class::<PyMarkdownParser>()?;

    m.add_function(wrap_pyfunction!(hello_word, m)?)?;
    Ok(())
}

#[pyfunction]
fn hello_word() -> String {
    "Hello World".to_string()
}

fn cast_anyhow(e: anyhow::Error) -> PyErr {
    PyException::new_err(format!("{}", e))
}

#[derive(FromPyObject, Clone)]
pub enum PyStringOrInt {
    #[pyo3(transparent, annotation = "str")]
    String(String),
    #[pyo3(transparent, annotation = "int")]
    Int(i32),
}

impl IntoPy<PyObject> for PyStringOrInt {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Self::String(value) => value.to_object(py),
            Self::Int(value) => value.to_object(py),
        }
    }
}

/* impl Clone for PyStringOrInt {
    fn clone(&self) -> Self {
        match self {
            Self::String(from) => Self::String(from.clone()),
            Self::Int(from) => Self::Int(from)
        }
    }
} */

#[pyclass(name = "Output")]
struct PyOutput {
    #[pyo3(get, set)]
    pub stderr: Vec<u8>,
    #[pyo3(get, set)]
    pub stdout: Vec<u8>,
    #[pyo3(get, set)]
    pub exit_code: PyStringOrInt,
}

#[pymethods]
impl PyOutput {
    #[new]
    fn new(stdout: &[u8], stderr: &[u8], exit_code: PyStringOrInt) -> Self {
        (&Output {
            stderr: stderr.to_vec().into(),
            stdout: stdout.to_vec().into(),
            exit_code: parse_exit_code(&exit_code),
        })
            .into()
    }
}

impl From<&Output> for PyOutput {
    fn from(output: &Output) -> Self {
        Self {
            stderr: (&output.stderr).into(),
            stdout: (&output.stdout).into(),
            exit_code: PyStringOrInt::String(output.exit_code.to_string()),
        }
    }
}

impl From<&PyOutput> for Output {
    fn from(output: &PyOutput) -> Self {
        Self {
            stderr: output.stderr.clone().into(),
            stdout: output.stdout.clone().into(),
            exit_code: parse_exit_code(&output.exit_code),
        }
    }
}

fn parse_exit_code(from: &PyStringOrInt) -> ExitStatus {
    match from {
        PyStringOrInt::Int(code) => ExitStatus::Code(*code),
        PyStringOrInt::String(status) => {
            if let Some(captures) = EXIT_CODE.captures(status) {
                ExitStatus::Code(captures.get(1).unwrap().as_str().parse::<i32>().unwrap())
            } else if let Some(captures) = EXIT_TIMEOUT.captures(status) {
                ExitStatus::Timeout(Duration::from_millis(
                    captures.get(1).unwrap().as_str().parse::<u64>().unwrap(),
                ))
            } else {
                ExitStatus::Unknown
            }
        }
    }
}

#[pyclass(name = "TestCase")]
struct PyTestCase {
    #[pyo3(get)]
    title: String,
    #[pyo3(get)]
    shell_expression: String,
    #[pyo3(get)]
    exit_code: i32,
    #[pyo3(get)]
    expectations: Vec<(String, Vec<u8>, bool, bool)>,
    #[pyo3(get)]
    line_number: usize,
    original: TestCase,
}

#[pymethods]
impl PyTestCase {
    #[new]
    fn new(
        title: &str,
        shell_expression: &str,
        exit_code: i32,
        expectations: Vec<String>,
        cram_compat: bool,
        line_number: Option<usize>,
    ) -> PyResult<Self> {
        let expectation_maker = new_expectation_maker(cram_compat);
        let expectations = expectations
            .iter()
            .map(|expression| {
                expectation_maker
                    .parse(expression)
                    .with_context(|| format!("parse expectation `{}`", expression))
                    .map_err(cast_anyhow)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let output_expectations = expectations
            .iter()
            .map(|expectation| expectation.unmake())
            .collect::<Vec<_>>();
        let line_number = line_number.unwrap_or(1);
        let testcase = TestCase {
            title: title.into(),
            shell_expression: shell_expression.into(),
            exit_code: Some(exit_code),
            config: if cram_compat {
                TestCaseConfig::default_cram()
            } else {
                TestCaseConfig::default_markdown()
            },
            expectations,
            line_number,
        };
        Ok(Self {
            title: title.into(),
            shell_expression: shell_expression.into(),
            exit_code,
            expectations: output_expectations,
            original: testcase,
            line_number,
        })
    }

    fn validate(&self, output: &PyOutput, location: Option<String>) -> PyResult<(bool, String)> {
        let output: Output = output.into();
        let renderer = PrettyMonochromeRenderer::default();
        let result = self.original.validate(&output);
        Ok((
            result.is_ok(),
            renderer
                .render(&[&Outcome {
                    testcase: self.original.clone(),
                    location,
                    output,
                    result,
                    escaping: Escaper::default(),
                    format: ParserType::Markdown,
                }])
                .map_err(cast_anyhow)?,
        ))
    }
}

impl From<&TestCase> for PyTestCase {
    fn from(testcase: &TestCase) -> PyTestCase {
        PyTestCase {
            title: testcase.title.to_owned(),
            shell_expression: testcase.shell_expression.to_owned(),
            exit_code: testcase.exit_code.unwrap_or(0),
            expectations: testcase
                .expectations
                .iter()
                .map(|expectation| expectation.unmake())
                .collect::<Vec<_>>(),
            line_number: testcase.line_number,
            original: testcase.clone(),
        }
    }
}

#[pyclass(name = "OutputStreamControl")]
#[derive(Clone)]
enum PyOutputStreamControl {
    Stdout,
    Stderr,
    Combined,
}

impl From<OutputStreamControl> for PyOutputStreamControl {
    fn from(value: OutputStreamControl) -> Self {
        match value {
            OutputStreamControl::Stdout => Self::Stdout,
            OutputStreamControl::Stderr => Self::Stderr,
            OutputStreamControl::Combined => Self::Combined,
        }
    }
}

impl From<PyOutputStreamControl> for OutputStreamControl {
    fn from(value: PyOutputStreamControl) -> Self {
        match value {
            PyOutputStreamControl::Stdout => Self::Stdout,
            PyOutputStreamControl::Stderr => Self::Stderr,
            PyOutputStreamControl::Combined => Self::Combined,
        }
    }
}

#[pyclass(name = "TestCaseWait")]
#[derive(Clone)]
pub struct PyTestCaseWait {
    #[pyo3(get)]
    pub timeout: usize,
    #[pyo3(get)]
    pub path: Option<String>,
}

impl From<TestCaseWait> for PyTestCaseWait {
    fn from(value: TestCaseWait) -> Self {
        Self {
            timeout: duration_to_millis(value.timeout),
            path: value.path.map(path_to_str),
        }
    }
}

#[pyclass(name = "TestCaseConfig")]
#[derive(Clone)]
struct PyTestCaseConfig {
    #[pyo3(get)]
    pub detached: Option<bool>,
    #[pyo3(get)]
    pub environment: HashMap<String, String>,
    #[pyo3(get)]
    pub keep_crlf: Option<bool>,
    #[pyo3(get)]
    pub output_stream: Option<PyOutputStreamControl>,
    #[pyo3(get)]
    pub skip_code: Option<u32>,
    #[pyo3(get)]
    pub timeout: Option<usize>,
    #[pyo3(get)]
    pub wait: Option<PyTestCaseWait>,
}

impl From<TestCaseConfig> for PyTestCaseConfig {
    fn from(value: TestCaseConfig) -> Self {
        Self {
            detached: value.detached,
            environment: value.environment,
            keep_crlf: value.keep_crlf,
            output_stream: value.output_stream.map(PyOutputStreamControl::from),
            skip_code: value.skip_code,
            timeout: value.timeout.map(duration_to_millis),
            wait: value.wait.map(PyTestCaseWait::from),
        }
    }
}

#[pyclass(name = "DocumentConfig")]
#[derive(Clone)]
struct PyDocumentConfig {
    #[pyo3(get)]
    pub append: Vec<String>,
    #[pyo3(get)]
    pub defaults: PyTestCaseConfig,
    #[pyo3(get)]
    pub language_markers: Vec<String>,
    #[pyo3(get)]
    pub prepend: Vec<String>,
    #[pyo3(get)]
    pub shell: Option<String>,
    #[pyo3(get)]
    pub total_timeout: Option<usize>,
}

impl From<DocumentConfig> for PyDocumentConfig {
    fn from(value: DocumentConfig) -> Self {
        Self {
            append: value.append.iter().map(path_to_str).collect::<Vec<_>>(),
            defaults: value.defaults.into(),
            language_markers: value.language_markers.clone(),
            prepend: value.prepend.iter().map(path_to_str).collect::<Vec<_>>(),
            shell: value.shell.map(path_to_str),
            total_timeout: value.total_timeout.map(duration_to_millis),
        }
    }
}

#[pyclass(name = "CramParser")]
struct PyCramParser(CramParser);

#[pymethods]
impl PyCramParser {
    #[new]
    fn new() -> Self {
        Self(CramParser::default_new(Arc::new(new_expectation_maker(
            true,
        ))))
    }

    fn parse(&self, text: &str) -> PyResult<(PyDocumentConfig, Vec<PyTestCase>)> {
        let (config, testcases) = self.0.parse(text).map_err(cast_anyhow)?;
        Ok((
            config.into(),
            testcases.iter().map(PyTestCase::from).collect::<Vec<_>>(),
        ))
    }
}

#[pyclass(name = "MarkdownParser")]
struct PyMarkdownParser(MarkdownParser);

#[pymethods]
impl PyMarkdownParser {
    #[new]
    fn new(languages: Vec<String>) -> Self {
        Self(MarkdownParser::new(
            Arc::new(new_expectation_maker(false)),
            &languages.iter().map(|s| s as &str).collect::<Vec<_>>(),
        ))
    }

    #[staticmethod]
    fn default() -> Self {
        Self(MarkdownParser::default_new(Arc::new(
            new_expectation_maker(false),
        )))
    }

    fn parse(&self, text: &str) -> PyResult<(PyDocumentConfig, Vec<PyTestCase>)> {
        let (config, testcases) = self.0.parse(text).map_err(cast_anyhow)?;
        Ok((
            config.into(),
            testcases.iter().map(PyTestCase::from).collect::<Vec<_>>(),
        ))
    }
}

fn new_expectation_maker(cram_compat: bool) -> ExpectationMaker {
    let mut registry = RuleRegistry::default();
    if cram_compat {
        registry.register(CramGlobRule::make, &["glob", "gl"]);
    }
    ExpectationMaker::new(registry)
}

fn path_to_str<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().into()
}

fn duration_to_millis(duration: Duration) -> usize {
    duration.as_millis() as usize
}
