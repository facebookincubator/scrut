#!/usr/bin/env bash
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

export FROM_A_CUSTOM_SHELL="yes"

exec /usr/bin/env bash "$@"
