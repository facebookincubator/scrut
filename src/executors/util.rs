use std::num::NonZeroUsize;
use std::thread::{self};

/// Default amount of parallel executions. This number often corresponds to the
/// amount of CPUs or computer has, but it may diverge in various cases.
pub fn default_parallel_count() -> usize {
    thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(1).expect("1 > 0"))
        .get()
}
