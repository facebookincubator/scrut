/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Session module - PTY session management, key input, output cleaning
//!
//! This is the foundation layer of the framework, providing:
//! - `TtySession` - Spawning and interacting with processes via PTY (cross-platform)
//! - `Keys` - Named key constants for input
//! - Output cleaning utilities for ANSI codes and TTY artifacts

mod expect;
mod keys;
mod output;
mod piped_session;
mod tty_session;

pub use expect::ExpectChain;
pub use keys::Keys;
pub use keys::key_name_to_sequence;
pub use output::clean_tty_output;
pub use output::strip_ansi_codes;
pub use piped_session::PipedOutput;
pub use piped_session::PipedSession;
pub use tty_session::DEFAULT_COLS;
pub use tty_session::DEFAULT_ROWS;
pub use tty_session::Spawn;
pub use tty_session::TtySession;
