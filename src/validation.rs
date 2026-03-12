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
//! Adding a new validation mode means extending these enums rather than
//! modifying every common type.

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
    /// JSON Schema mode: validate command output against a JSON Schema.
    JsonSchema(JsonSchemaBody),
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
            (Self::JsonSchema(a), Self::JsonSchema(b)) => a == b,
            _ => false,
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
            // JSON Schema body is not serialized as expectations
            Self::JsonSchema(_) => serializer.serialize_none(),
        }
    }
}

/// Body for output-mode test cases.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutputBody {
    /// The expectations that describe the expected output of the execution.
    pub expectations: Vec<Expectation>,
}

/// Body for JSON Schema validation mode test cases.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonSchemaBody {
    /// Raw YAML/JSON source text of the JSON Schema definition.
    pub schema_source: String,
}

/// The kind of JSON Schema validation failure.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonSchemaFailureKind {
    /// The schema definition cannot be parsed.
    InvalidSchema,
    /// The command output is not valid JSON.
    InvalidJson,
    /// The command output doesn't conform to the schema.
    ValidationErrors,
}

/// A JSON Schema validation failure with context.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonSchemaFailure {
    /// The kind of failure.
    pub kind: JsonSchemaFailureKind,
    /// Human-readable error messages.
    pub errors: Vec<String>,
    /// The actual command output.
    pub output: String,
    /// The schema source that was used.
    pub schema_source: String,
}

/// Mode-specific validation failure, replacing separate `MalformedOutput` and
/// `InteractiveFailed` variants on [`crate::testcase::TestCaseError`].
#[derive(Clone, Debug)]
pub enum ValidationFailure {
    /// Output lines did not match expectations.
    MalformedOutput(Diff),
    /// JSON Schema validation failed.
    JsonSchemaFailed(JsonSchemaFailure),
}

impl PartialEq for ValidationFailure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MalformedOutput(a), Self::MalformedOutput(b)) => a == b,
            (Self::JsonSchemaFailed(a), Self::JsonSchemaFailed(b)) => a == b,
            _ => false,
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
            Self::JsonSchemaFailed(failure) => {
                let kind = match failure.kind {
                    JsonSchemaFailureKind::InvalidSchema => "json_schema_invalid_schema",
                    JsonSchemaFailureKind::InvalidJson => "json_schema_invalid_json",
                    JsonSchemaFailureKind::ValidationErrors => "json_schema_validation_errors",
                };
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("kind", kind)?;
                map.serialize_entry("errors", &failure.errors)?;
                map.serialize_entry("output", &failure.output)?;
                map.end()
            }
        }
    }
}
