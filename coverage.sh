#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

set -e
#set -x

export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="coverage/profiles/scrut-%p-%m.profraw"

# cleanup & init
[ -d coverage ] && rm -rf coverage
mkdir -p coverage/profiles

# create binary
cargo build --verbose

# run tests to create coverage
cargo test --all-features --verbose -- --include-ignored

# render coverage
grcov . --binary-path ./target/debug/ \
    -s . \
    -t html \
    --branch \
    --ignore-not-existing \
    --ignore "/*" \
    -o ./coverage/output/

echo "Output: file://$(pwd)/coverage/output/index.html"
if [ $(uname) == "Darwin" ]; then
    open coverage/output/index.html
fi
