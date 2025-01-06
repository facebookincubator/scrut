/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

extern crate scrut;

mod commands;
mod fb_main_common;
mod utils;

use anyhow::Result;
use cli::ExitCode;
use fb_main_common::main_impl;
use fb_main_common::Args;
use fbinit::FacebookInit;

#[cli::main("scrut", usage_logging(enabled = false))]
pub fn main(fb: FacebookInit, args: Args) -> Result<ExitCode> {
    main_impl(fb, args)
}
