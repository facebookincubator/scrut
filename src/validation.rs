/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Discriminated unions for validation mode-specific data.
//!
//! These types decouple mode-specific test body and failure data from the
//! common [`crate::testcase::TestCase`] and [`crate::output::Output`] structs.
//! Adding a new validation mode (e.g., `json_schema`) means extending these
//! enums rather than modifying every common type.

use serde::Serialize;
use serde::ser::SerializeMap;

use crate::diff::Diff;
use crate::expectation::Expectation;

/// Mode-specific test body, replacing flat `expectations` + `interactive_directives`
/// fields on [`crate::testcase::TestCase`].
#[derive(Clone, Debug)]
pub enum ValidationBody {
    /// Output mode: line-by-line expectations compared against command output.
    Output(OutputBody),
}

impl Default for ValidationBody {
    fn default() -> Self {
        Self::Output(OutputBody::default())
    }
}

impl PartialEq for ValidationBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Output(a), Self::Output(b)) => a == b,
        }
    }
}

impl Serialize for ValidationBody {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Output(body) => body.expectations.serialize(serializer),
        }
    }
}

/// Body for output-mode test cases.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutputBody {
    /// The expectations that describe the expected output of the execution.
    pub expectations: Vec<Expectation>,
}

/// Mode-specific validation failure, replacing separate `MalformedOutput` and
/// `InteractiveFailed` variants on [`crate::testcase::TestCaseError`].
#[derive(Clone, Debug)]
pub enum ValidationFailure {
    /// Output lines did not match expectations.
    MalformedOutput(Diff),
}

impl PartialEq for ValidationFailure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MalformedOutput(a), Self::MalformedOutput(b)) => a == b,
        }
    }
}

impl Serialize for ValidationFailure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::MalformedOutput(diff) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("kind", "malformed_output")?;
                map.serialize_entry("diff", &diff.lines)?;
                map.end()
            }
        }
    }
}
