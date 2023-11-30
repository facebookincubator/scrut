/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;
use std::time::Duration;

/// Aggregate of all required input for an execution that is passed to
/// [`super::executor::Executor::execute_all`] calls.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Execution {
    /// A shell expression that is to be executed
    pub expression: String,

    /// Optional list of environment variables
    pub environment: Option<HashMap<String, String>>,

    /// Optional timeout that limits the maximum execution length
    pub timeout: Option<Duration>,
}

impl Execution {
    /// Create a new Execution only from a shell expression, with empty
    /// environment and no timeout
    pub fn new(expression: &str) -> Self {
        Execution {
            expression: expression.to_string(),
            ..Default::default()
        }
    }

    /// Builder setter for expression
    pub fn expression(mut self, expression: &str) -> Self {
        self.expression = expression.into();
        self
    }

    /// Builder setter for environment variables
    pub fn environment(mut self, keys_values: &[(&str, &str)]) -> Self {
        self.environment = Some(HashMap::from_iter(
            keys_values
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        ));
        self
    }

    /// Builder setter for timeout
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }
}
