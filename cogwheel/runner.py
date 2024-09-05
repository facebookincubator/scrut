# (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.

# pyre-strict
from __future__ import annotations

import dataclasses
import json
import os
import subprocess
import sys
from argparse import ArgumentParser
from dataclasses import dataclass
from pathlib import Path

from clifoundation.scrut.buck.scruttest import generate_test_method_name

from windtunnel.cogwheel.failure import (
    AssertionFailure,
    FailureHandler,
    InfrastructureFailure,
    UserSetupFailure,
    UserSkippedFailure,
)
from windtunnel.cogwheel.lib.logging import FrameworkLogger, TestLogger
from windtunnel.cogwheel.result import CogwheelTestSuiteResult
from windtunnel.cogwheel.result_if.ttypes import Result
from windtunnel.cogwheel.test import cogwheel_test, CogwheelTest


SCRUT_BINARY: str = "/usr/local/bin/scrut"
SCRUT_EXTENSIONS: set[str] = {".md", ".markdown", ".t", ".cram"}
WORKLOAD_ENVVAR: str = "SERVICELAB_WORKLOAD"
SCRUT_CONFIG_FILE: str = "scrut_config.json"


@dataclass
class CogwheelScrutTestConfig:
    args: list[str]
    srcs: list[str]
    prepend_srcs: list[str]
    append_srcs: list[str]
    env: dict[str, str]


class CogwheelScrutTestFailureHandler(FailureHandler):

    def handleTestFailure(
        self, e: Exception, name: str, result: CogwheelTestSuiteResult
    ) -> None:
        if isinstance(e, AssertionFailure):
            FrameworkLogger.error(f"An assertion failed: {e}")

        if isinstance(e, InfrastructureFailure):
            status = Result.INFRA_FAILURE
        elif isinstance(e, UserSkippedFailure):
            result.skipped.append(name)
            status = Result.SKIPPED
        else:
            status = Result.FAILURE

        result.setTestStatus(
            test_name=name,
            status=status,
            type=type(e).__name__,
            message=str(e),
            stacktrace=None,
        )

    def handleTestsuiteFailure(
        self, e: Exception, result: CogwheelTestSuiteResult
    ) -> None:
        if isinstance(e, InfrastructureFailure):
            status = Result.INFRA_FAILURE
        elif isinstance(e, UserSetupFailure):
            status = Result.FAILURE
        elif isinstance(e, UserSkippedFailure):
            status = Result.SKIPPED
        else:
            status = Result.SETUP_FAILURE

        result.setTestsuiteStatus(
            status=status,
            type=type(e).__name__,
            message=None,
            stacktrace=None,
            is_status_for_main_test=True,
        )
        result.setMainTestStatus(status)


class CogwheelScrutTest(CogwheelTest):

    def __init__(self, config: CogwheelScrutTestConfig) -> None:
        super().__init__(handler=CogwheelScrutTestFailureHandler())
        self._config = config
        TestLogger.info(
            f"Initialize workload {_workload_name()} with {json.dumps(dataclasses.asdict(config))}"
        )
        self._register_tests()

    def _register_tests(self) -> None:
        """
        Iterate all test files and register them as a Python test method
        """

        for src in sorted(self._config.srcs):
            ext = Path(src).suffix
            if ext not in SCRUT_EXTENSIONS:
                continue
            self._setup_test(src)

        pass

    def _setup_test(self, path: str) -> None:
        """
        Create a callback test and register with `cogwheel_test` decorator
        """

        def call_scrut_test(self: CogwheelScrutTest) -> None:
            self._run_test(path)

        name = generate_test_method_name(Path(path))
        TestLogger.info(f"Setup test {path} as {name}")
        # pyre-ignore[16]
        call_scrut_test.__name__ = name
        cogwheel_test(call_scrut_test)

    def _run_test(self, path: str) -> None:
        """
        Execute a test runnig `scrut test ... <test-file>
        """

        args = self._build_args()
        TestLogger.info(f"Run test {path} with args {json.dumps(args)}")
        stdout, stderr, code = self._run(
            [
                SCRUT_BINARY,
                "test",
                "--log-level=debug",
                *args,
                path,
            ]
        )
        if code == 0:
            TestLogger.info(
                f"Test {path} succeded with exit code {code}\n\nSTDOUT:\n{stdout}\n\nSTDERR:\n{stderr}\n"
            )
        self.assertEqual(
            0,
            code,
            f"Test {path} failed with exit code {code}\n\nSTDOUT:\n{stdout}\n\nSTDERR:\n{stderr}\n",
        )

    def _build_args(self) -> list[str]:
        """
        Create list of parameters for `scrut test` execution
        """
        args = self._config.args.copy()
        for param, srcs in {
            "--prepend-test-file-paths": self._config.prepend_srcs,
            "--append-test-file-paths": self._config.append_srcs,
        }.items():
            if not srcs:
                continue
            args.extend([param, " ".join(srcs)])
        return args

    def _run(self, cmd: str | list[str]) -> tuple[str, str, int | None]:
        """
        Excecute a command and return the output
        """
        TestLogger.info(f"Run command {json.dumps(cmd)}")
        result = subprocess.run(
            cmd,
            capture_output=True,
            env=self._config.env,
            cwd=self._test_srcs_directory(),
        )
        return (
            result.stdout.decode("utf-8"),
            result.stderr.decode("utf-8"),
            result.returncode,
        )

    def _test_srcs_directory(self) -> str:
        """
        Path to the directory where the test files are located
        """
        return f"{_harness_directory(self.get_package_path())}/test_srcs"


def main() -> None:
    parser = ArgumentParser("CogwheelScrutTest harness", add_help=False)
    parser.add_argument("--package-path")
    args, _ = parser.parse_known_args()

    config_file = os.path.join(_harness_directory(args.package_path), SCRUT_CONFIG_FILE)
    TestLogger.info(f"Loading scrut config from file {config_file}")
    with open(config_file) as f:
        config = json.load(f)
        TestLogger.info(f"Loaded scrut config {json.dumps(config)}")
        CogwheelScrutTest(
            config=CogwheelScrutTestConfig(**config),
        ).main()


def _harness_directory(package_path: str | None) -> str:
    """
    Returns the path to the directory where the test harness is located
    """
    if not package_path:
        package_path = "/packages"
    return f"{package_path}/{_workload_name()}_test_harness"


def _workload_name() -> str:
    """
    Returns the name of the workload that has the shape `cogwheel_scrut_<oncall>_<name>`.
    """
    try:
        # environment variable is only available in remote executions
        return os.environ[WORKLOAD_ENVVAR]
    except KeyError:
        # fallback to using the name of the par file that has the format:
        #  /packages/<workload_name>_test_harness/<workload_name>.par
        # this is needed in local runs where the environment variable is notset
        return os.path.basename(sys.argv[0]).removesuffix(".par")


if __name__ == "__main__":
    main()  # pragma: no cover
