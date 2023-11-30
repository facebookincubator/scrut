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
use std::io;
use std::io::stdout;
use std::io::IsTerminal;
use std::process::ExitCode;

use clap::Parser;
use commands::root::Commands;
use commands::root::GlobalParameters;
use commands::test::ValidationFailedError;
use tracing::error;
use tracing::Level;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::FmtSubscriber;

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
    init_logging();
    let result = Args::parse().commands.run();

    if let Err(err) = result {
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

const DEFAULT_LOG_LEVEL: Level = Level::WARN;

fn init_logging() {
    let filter = EnvFilter::builder()
        .with_default_directive(Directive::from(DEFAULT_LOG_LEVEL))
        .from_env()
        .expect("failed to create default EnvFilter");

    FmtSubscriber::builder()
        .with_ansi(stdout().is_terminal())
        .with_writer(io::stderr)
        .with_env_filter(filter)
        .init()
}
