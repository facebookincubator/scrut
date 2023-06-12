#!/bin/bash

if [ "${SCRUT_BIN}" == "" ]; then
    echo "environment variable SCRUT_BIN with path to binary must be set"
    exit ${NO_SCRUT_EXIT_CODE:-1}
fi

echo OK
