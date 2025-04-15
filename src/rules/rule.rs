/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fmt::Debug;
use std::fmt::Display;

use anyhow::Result;
use serde::Serialize;
use serde::ser::SerializeMap;

use crate::escaping::Escaper;

/// Rule implements the line-level comparisons of [`crate::expectation::Expectation`]s
pub trait Rule: RuleClone + Debug + Send {
    /// What kind (type name) the rule has
    fn kind(&self) -> &'static str;

    /// Whether the rule matches - can be applied to - the given line
    fn matches(&self, line: &[u8]) -> bool;

    /// Decompose the rule into components from which it can be re-made
    fn unmake(&self) -> (String, Vec<u8>);

    /// The string representation of the Rule as it would be written in
    /// a test document
    fn to_expression_string(&self, optional: bool, multiline: bool, escaper: &Escaper) -> String {
        let (quantifier, equal_quantifier) = if optional {
            if multiline {
                ("*", " (*)")
            } else {
                ("?", " (?)")
            }
        } else if multiline {
            ("+", " (+)")
        } else {
            ("", "")
        };
        let (kind, expression) = self.unmake();
        let rendered = escaper.escaped_printable(&expression);
        if kind == "equal" {
            if escaper.has_unprintable(&expression) {
                format!("{rendered} (escaped{quantifier})")
            } else {
                format!("{rendered}{equal_quantifier}")
            }
        } else {
            format!("{rendered} ({kind}{quantifier})")
        }
    }
}

impl Display for Box<dyn Rule> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (kind, expression) = self.unmake();
        let expression = Escaper::Unicode.escaped_printable(&expression);
        write!(f, "{kind}::{expression}")
    }
}

impl PartialEq<Box<dyn Rule>> for Box<dyn Rule> {
    fn eq(&self, other: &Box<dyn Rule>) -> bool {
        format!("{self}") == format!("{other}")
    }
}

impl Clone for Box<dyn Rule> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Serialize for Box<dyn Rule> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (kind, expression) = self.unmake();
        let expression = Escaper::Unicode.escaped_printable(&expression);
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kind", &kind)?;
        map.serialize_entry("expression", &expression)?;
        map.end()
    }
}

// serialize_trait_object!(Rule);

pub trait RuleClone {
    fn clone_box(&self) -> Box<dyn Rule>;
}

impl<T: 'static + Rule + Clone> RuleClone for T {
    fn clone_box(&self) -> Box<dyn Rule> {
        Box::new(self.clone())
    }
}

/// Trait
pub trait RuleMaker {
    fn make(expression: &str) -> Result<Box<dyn Rule>>;
}

/// Constructor function for [`Rule`] implementations
pub type MakeRule = fn(&str) -> Result<Box<dyn Rule>>;
