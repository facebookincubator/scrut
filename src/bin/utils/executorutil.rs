/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::Path;

use anyhow::Result;
use moon_cram::executors::bash_runner::BashRunner;
use moon_cram::executors::bash_script_executor::BashScriptExecutor;
use moon_cram::executors::executor::Executor;
use moon_cram::executors::stateful_executor::StatefulExecutor;

pub(crate) fn make_executor(shell: &Path, cram_compat: bool) -> Result<Box<dyn Executor>> {
    Ok(if cram_compat {
        Box::new(BashScriptExecutor::new(shell))
    } else {
        Box::new(StatefulExecutor::new(BashRunner::stateful_generator(shell)))
    })
}
