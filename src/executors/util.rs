/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::num::NonZeroUsize;
use std::thread::{self};

/// Default amount of parallel executions. This number often corresponds to the
/// amount of CPUs or computer has, but it may diverge in various cases.
pub fn default_parallel_count() -> usize {
    thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(1).expect("1 > 0"))
        .get()
}
