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
use std::process::ExitCode;

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap_complete::generate;
use clap_complete::Shell;
use commands::root::GlobalParameters;
use commands::test::ValidationFailedError;
use tracing::error;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

/// Arguments for the completions subcommand
#[derive(Debug, Parser)]
struct CompletionsArgs {
    /// The shell to generate completions for
    #[clap(value_enum, id = "target_shell")]
    target_shell: Shell,
}

/// All CLI subcommands
#[derive(Debug, Subcommand)]
enum CliCommands {
    Create(commands::create::Args),
    Test(commands::test::Args),
    Update(commands::update::Args),
    /// Generate shell completions
    Completions(CompletionsArgs),
}

#[derive(Debug, Parser)]
#[clap(about = "A testing toolkit to scrutinize CLI applications", version = VERSION)]
struct Args {
    #[clap(subcommand)]
    command: CliCommands,

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

    let result = match app.command {
        CliCommands::Create(cmd) => cmd.run(),
        CliCommands::Test(cmd) => cmd.run(),
        CliCommands::Update(cmd) => cmd.run(),
        CliCommands::Completions(args) => {
            let mut cmd = Args::command();
            generate(args.target_shell, &mut cmd, "scrut", &mut io::stdout());
            return ExitCode::SUCCESS;
        }
    };

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
