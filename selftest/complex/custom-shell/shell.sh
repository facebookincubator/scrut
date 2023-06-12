#!/usr/bin/env bash

export FROM_A_CUSTOM_SHELL="yes"

exec /usr/bin/env bash "$@"
