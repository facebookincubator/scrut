/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

/// Command Line Application Testing - with no fuzz
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate lazy_static;

pub mod config;
pub mod debug;
pub mod diff;
pub mod escaping;
pub mod executors;
pub mod expectation;
pub mod generators;
pub mod newline;
pub mod outcome;
pub mod output;
pub mod parsers;
pub mod renderers;
pub mod rules;
pub mod signal;
pub mod testcase;
