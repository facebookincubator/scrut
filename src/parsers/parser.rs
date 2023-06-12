use std::fmt::Display;

use anyhow::Result;
use clap::ValueEnum;

use crate::testcase::TestCase;

/// A Parser extracts one or more [`crate::testcase::TestCase`]s from a provided text
pub trait Parser {
    /// Returns all testcases found in the provided text
    fn parse(&self, tests: &str) -> Result<Vec<TestCase>>;
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum ParserType {
    Markdown,
    Cram,
}

impl ParserType {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Cram => "t",
            Self::Markdown => "md",
        }
    }
}

impl Display for ParserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cram => "cram",
                Self::Markdown => "markdown",
            }
        )
    }
}

impl TryFrom<String> for ParserType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &value as &str {
            "markdown" | "md" => Ok(Self::Markdown),
            "cram" => Ok(Self::Cram),
            _ => Err(format!("Unsupported parser format `{value}`")),
        }
    }
}
