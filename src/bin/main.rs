/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

extern crate scrut;

mod commands;
mod utils;

use std::env;
use std::process::ExitCode;

use clap::Parser;
use commands::root::Commands;
use commands::root::GlobalParameters;
use commands::test::ValidationFailedError;
use tracing::error;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

#[derive(Debug, Parser)]
#[clap(about = "A testing toolkit to scrutinize CLI applications", version = VERSION)]
struct Args {
    #[clap(subcommand)]
    commands: Commands,

    #[clap(flatten)]
    global: GlobalParameters,
}

pub fn main() -> ExitCode {
    // init_logging();
    let app = Args::parse();

    #[cfg(feature = "logging")]
    if let Err(err) = app.global.init_logging() {
        panic!("Failed to initialize logging: {:?}", err);
    }

    if let Err(err) = app.commands.run() {
        match err.downcast_ref::<ValidationFailedError>() {
            Some(_) => 50.into(),
            None => {
                error!("Error: {:?}", err);
                1.into()
            }
        }
    } else {
        ExitCode::SUCCESS
    }
}
