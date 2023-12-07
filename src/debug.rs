/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tracing::debug;

pub fn print_bytewise(name: &str, bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        debug!(name, i, "{:02x} (`{}`)", *b, *b as char)
    }
}

pub fn compare_bytewise(name: &str, s1: &[u8], s2: &[u8]) {
    let mut index = 0;
    if s1.len() >= s2.len() {
        for (v1, v2) in s1.iter().zip(s2.iter().chain(std::iter::repeat(&0))) {
            debug!(
                name,
                index, "{:02x} vs {:02x} (`{}` vs `{}`)", v1, *v2, *v1 as char, *v2 as char
            );
            index += 1;
        }
    } else {
        for (v2, v1) in s2.iter().zip(s1.iter().chain(std::iter::repeat(&0))) {
            debug!(
                name,
                index, "{:02x} vs {:02x} (`{}` vs `{}`)", v1, *v2, *v1 as char, *v2 as char
            );
            index += 1;
        }
    }
}

#[macro_export]
macro_rules! debug_bytewise {
    ($name:expr, $bytes:expr) => {{ $crate::debug::print_bytewise($name, $bytes) }};
    ($name:expr, $left:expr, $right:expr) => {{ $crate::debug::compare_bytewise($name, $left, $right) }};
}
