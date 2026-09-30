/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Provides the version that `scrut --version` reports: the `git describe`
//! output of the checkout being built, or the crate version outside of one.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let version = git_describe().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    let out_dir = env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
    let dest_path = Path::new(&out_dir).join("version.rs");
    fs::write(&dest_path, format!("const VERSION: &str = {version:?};\n"))
        .expect("write version to file");
    println!("cargo:rerun-if-changed=build.rs");
}

fn git_describe() -> Option<String> {
    let output = Command::new("git").arg("describe").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!version.is_empty()).then_some(version)
}
