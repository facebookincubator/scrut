//! This module is concerned with generating Markdown or Cram syntax, that is
//! used in creating or updating test files.
//!
//! The main interfaces are specified in the following traits:
//! - [`generator::TestCaseGenerator`], encoding single [`crate::testcase::TestCase`]s,
//!   used in creation of new test files
//! - [`generator::UpdateGenerator`], updating all [`crate::testcase::TestCase`]s
//!   that are found in an existing test file
//!
//! These traits are implemented as
//! - Markdown syntax: [`markdown::MarkdownTestCaseGenerator`],
//!   [`markdown::MarkdownUpdateGenerator`]
//! - Cram syntax: [`cram::CramTestCaseGenerator`], [`cram::CramUpdateGenerator`]

pub mod cram;
pub mod generator;
pub mod markdown;
pub mod outcome;
