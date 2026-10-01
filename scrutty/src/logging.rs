/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Logging module - Structured logging with session output flushing
//!
//! This module provides:
//! - `log_message()` - Log a message with automatic session output flushing
//! - `SessionRegistry` - Track active sessions for periodic output display
//! - `LogConfig` - Configuration for logging behavior

mod logger;
mod registry;

pub use logger::LogConfig;
pub use logger::log_message;
pub use logger::log_message_with_config;
pub use registry::SessionRegistry;
