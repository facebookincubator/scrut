/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::Result;

use crate::diff::Diff;
use crate::outcome::Outcome;
use crate::testcase::TestCaseError;

/// Renderer translate errors from validating [`crate::testcase::TestCase`]s into
/// text that allows humans to understand how the actual output is different
/// from the expected output
pub trait Renderer {
    fn render(&self, outcomes: &[&Outcome]) -> Result<String>;
}

/// This seems like a good idea atm? Assuming there will be more renderers Maybe not ..
pub(super) trait ErrorRenderer: Renderer {
    fn render_error(&self, err: &TestCaseError, outcome: &Outcome) -> Result<String> {
        match err {
            TestCaseError::MalformedOutput(diff) => self.render_malformed_output(outcome, diff),
            TestCaseError::InvalidExitCode { actual, expected } => {
                self.render_invalid_exit_code(outcome, *actual, *expected)
            }
            TestCaseError::InternalError(err) => self.render_delegated_error(outcome, err),
            TestCaseError::Skipped => self.render_skipped(outcome),
        }
    }

    fn render_invalid_exit_code(
        &self,
        outcome: &Outcome,
        actual: i32,
        expected: i32,
    ) -> Result<String>;

    fn render_delegated_error(&self, outcome: &Outcome, err: &anyhow::Error) -> Result<String>;

    fn render_malformed_output(&self, outcome: &Outcome, diff: &Diff) -> Result<String>;

    fn render_skipped(&self, outcome: &Outcome) -> Result<String>;
}
