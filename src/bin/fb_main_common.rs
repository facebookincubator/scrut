/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use cli::ExitCode;
use fbinit::FacebookInit;
use scrut::parsers::markdown::MarkdownParserError;
use tracing::error;

use crate::commands::root::Commands;
use crate::commands::root::GlobalParameters;
use crate::commands::test::ValidationFailedError;

#[derive(Debug, Parser)]
#[clap(
    about = "A testing toolkit to scrutinize CLI applications",
    author = "clifoundation"
)]
pub struct Args {
    #[clap(subcommand)]
    commands: Commands,

    #[clap(flatten)]
    global: GlobalParameters,
}

/// Implemented here because it's only used in `fb-main*.rs`, and so
/// there's little point in exposing it to the OSS version.
pub fn is_user_error(err: &anyhow::Error) -> bool {
    if let Some(MarkdownParserError::MissingLanguageSpecifier { .. }) =
        err.downcast_ref::<MarkdownParserError>()
    {
        return true;
    }
    false
}

pub fn main_impl(_fb: FacebookInit, args: Args) -> Result<ExitCode> {
    if let Err(err) = args.commands.run() {
        if err.downcast_ref::<ValidationFailedError>().is_some() {
            return Ok(ExitCode::from(50));
        }
        if is_user_error(&err) {
            error!("{:?}", err);
            Ok(ExitCode::from(1))
        } else {
            Err(anyhow!(err))
        }
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
