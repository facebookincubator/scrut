//! This module is concerned with parsing test files and extracting
//! [`crate::testcase::TestCase`]s from them. Part of the parsing is also the
//! rejection of invalid formatted test files.
//!
//! The [`parser::Parser`] trait provides the interface, that works on the
//! content files. Currently two implementations are supported:
//! - Markdown file syntax: [`markdown::MarkdownParser`]
//! - Cram file syntax: [`cram::CramParser`]

pub mod cram;
pub(super) mod line_parser;
pub mod markdown;
pub mod parser;
