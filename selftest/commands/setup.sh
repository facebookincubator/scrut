#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

if [ "${SCRUT_BIN}" == "" ]; then
    echo "environment variable SCRUT_BIN with path to binary must be set"
    exit ${NO_SCRUT_EXIT_CODE:-1}
fi

echo OK
