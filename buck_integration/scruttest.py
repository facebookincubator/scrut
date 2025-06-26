# (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.

# pyre-strict

# ----------------------------------
# This library exports shared functionality for Buck macros:
# - `scrut_test`
# - `buck_scrut_test`
# - `coghweel_scrut_test` (planned)
# ----------------------------------


import os
import re
from pathlib import Path


class ScrutAssertionError(AssertionError):
    output: str
    cmd: object

    def __init__(
        self, *args: object, output: str, cmd: object, **kwargs: object
    ) -> None:
        self.cmd = cmd
        self.output = output
        super().__init__(*args, **kwargs)

    def __str__(self) -> str:
        return "validation failed"

    def render(self) -> str:
        cmd = " ".join(self.cmd) if isinstance(self.cmd, list) else self.cmd
        return f"{cmd}:\n\n{self.output}"


def _path_component_to_method_name(path: str) -> str:
    name = re.sub("[^a-z0-9_]", "_", path.lower())
    name = re.sub("__+", "_", name)
    return name.strip("_")


def generate_test_method_name(path: Path) -> str:
    method_name = _path_component_to_method_name(path.name)
    if len(parents := path.parent.name.split(os.sep)) > 0:
        parent = _path_component_to_method_name(parents.pop())
        method_name = f"{parent}_{method_name}"
    if not method_name.startswith("test_"):
        method_name = f"test_{method_name}"
    return method_name
