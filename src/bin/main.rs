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

use clap::CommandFactory;
use clap::Parser;
use clap_complete::aot::generate;
use clap_complete::aot::Shell;
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

fn generate_completion(completion_value: &str) -> ExitCode {
    let shell = match completion_value {
        "bash_source" => Shell::Bash,
        "elvish_source" => Shell::Elvish,
        "fish_source" => Shell::Fish,
        "powershell_source" => Shell::PowerShell,
        "zsh_source" => Shell::Zsh,
        _ => {
            eprintln!(
                "Error: Invalid value for _SCRUT_COMPLETE: '{}'\n\
                Valid values: bash_source, elvish_source, fish_source, powershell_source, zsh_source",
                completion_value
            );
            return 1.into();
        }
    };

    let mut command = Args::command();
    generate(shell, &mut command, "scrut", &mut std::io::stdout());
    ExitCode::SUCCESS
}

pub fn main() -> ExitCode {
    if let Ok(completion_value) = env::var("_SCRUT_COMPLETE") {
        return generate_completion(&completion_value);
    }

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
