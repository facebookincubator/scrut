/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

/**
 * Build file that injects Version and Git build information
 */
extern crate vergen;

use std::env;
use std::fmt::Error;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use vergen::vergen;
use vergen::Config;

fn main() {
    let content = vergen(Config::default())
        .map(|_| "const VERSION: &str = env!(\"VERGEN_GIT_SEMVER\");".to_string())
        .or_else(|_| {
            // building with a version determined from git is neat - but it must
            // not be a blocker. If the that fails, then fallback to the current
            // timestamp is fine enough
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("get current timestamp");
            let timestamp = format!("const VERSION: &str = \"{}\";", timestamp.as_secs());
            Ok::<String, Error>(timestamp)
        })
        .expect("generate version information");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version.rs");
    fs::write(&dest_path, &content).expect("write version to file");
    println!("cargo:rerun-if-changed=build.rs");
}
