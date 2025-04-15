/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::borrow::Cow;

use anyhow::Context;
use anyhow::Result;
use dialoguer::Confirm;
use dialoguer::console::Term;
use dialoguer::console::strip_ansi_codes;
use dialoguer::theme::ColorfulTheme;
use dialoguer::theme::SimpleTheme;
use dialoguer::theme::Theme;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use tracing::info;
use tracing::warn;

/// Prompt user to reply with YES or NO
pub(crate) fn confirm(question: &str, default: bool, no_color: bool) -> Result<bool> {
    let theme: Box<dyn Theme> = if no_color {
        Box::new(SimpleTheme {})
    } else {
        Box::new(ColorfulTheme::default())
    };
    let confirmed = Confirm::with_theme(&*theme)
        .with_prompt(question)
        .default(default)
        .show_default(true)
        .interact_on(&Term::stderr())
        .map_err(anyhow::Error::new)?;
    Ok(confirmed)
}

pub(crate) fn progress_bar(size: u64) -> Result<ProgressBar> {
    let len: usize = size.to_string().len();
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::with_template(&format!(
            "[{{pos:>{}}}/{{len:>{}}}]▕{{bar:.blue}}▏{{msg}}",
            len, len,
        ))
        .context("create progress bar style")?
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    // pb.enable_steady_tick(std::time::Duration::from_millis(100));
    Ok(pb)
}

/// A wrapper that exports a subset of the `ProgressBar` API. It can be instantiated to use an
/// actual `ProgressBar` or use log messages instead.
///
/// Use-case: We want to show progress bars in the CLI but we don't want to show them if user
///           wants increased log messages (info-level or above)
/// Use-case: Testing. ProgressBar does not create well testable output, logs do.
pub(crate) struct ProgressWriter {
    pb: Option<ProgressBar>,
    no_color: bool,
}

impl ProgressWriter {
    pub fn try_new(size: u64, show_progress_bar: bool, no_color: bool) -> Result<Self> {
        let pb = if show_progress_bar {
            Some(progress_bar(size)?)
        } else {
            None
        };
        Ok(Self { pb, no_color })
    }

    pub fn println<S: AsRef<str>>(&self, msg: S) {
        let msg = self.render(msg.as_ref());
        if let Some(pb) = &self.pb {
            pb.println(msg);
        } else if !msg.is_empty() {
            if is_warn_or_error_message(&msg) {
                warn!("{}", msg);
            } else {
                info!("{}", msg);
            }
        }
    }

    pub fn inc(&self, delta: u64) {
        if let Some(pb) = &self.pb {
            pb.inc(delta);
        }
    }

    pub fn set_message<S: AsRef<str>>(&self, msg: S) {
        let msg = self.render(msg.as_ref()).into_owned();
        if let Some(pb) = &self.pb {
            if !is_in_scrut_test() {
                pb.set_message(msg);
            }
        } else {
            info!("{}", &msg);
        }
    }

    pub fn finish_and_clear(&self) {
        if let Some(pb) = &self.pb {
            pb.finish_and_clear();
        }
    }

    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        if let Some(pb) = &self.pb {
            pb.suspend(f)
        } else {
            f()
        }
    }

    fn render<'a>(&self, msg: &'a str) -> Cow<'a, str> {
        if self.no_color {
            strip_ansi_codes(msg)
        } else {
            Cow::from(msg)
        }
    }
}

fn is_warn_or_error_message(msg: &str) -> bool {
    msg.contains("❌") || msg.contains("⌛️")
}

fn is_in_scrut_test() -> bool {
    std::env::var("SCRUT_TEST").is_ok()
}

/// Get the current log level that has been set by the user
pub(crate) fn get_log_level() -> tracing::Level {
    tracing::level_filters::LevelFilter::current()
        .into_level()
        .unwrap_or(tracing::Level::WARN)
}
