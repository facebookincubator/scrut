#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

if [ "${MOON_CRAM_BIN}" == "" ]; then
    echo "environment variable MOON_CRAM_BIN with path to binary must be set"
    exit ${NO_MOON_CRAM_EXIT_CODE:-1}
fi

echo OK
