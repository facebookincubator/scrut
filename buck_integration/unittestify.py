# (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.

# ----------------------------------
# !THIS FILE IS EXECUTED, THIS IS NOT A LIBRARY!
# ----------------------------------
# This is largly stolen from our Cram code in //third-party/cram/unittestify.py
# ----------------------------------
# Translate Scrut tests to standard Python unittest tests.
# ----------------------------------

from __future__ import annotations

import os
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Callable

from . import scruttest


class ScrutTests(unittest.TestCase):
    @classmethod
    def auto_generate_tests(cls) -> None:
        """Add test files as methods"""

        env = os.environ.copy()

        try:
            scrut_args = env.pop("SCRUT_ARGS").split()
        except KeyError as err:
            raise Exception("Required SCRUT_ARGS env var not set") from err

        try:
            scrut_location = env.pop("SCRUT_LOCATION")
        except KeyError as err:
            raise Exception("Required SCRUT_LOCATION env var not set") from err

        try:
            scrut_tests_dir = env.pop("SCRUT_TESTS_DIR")
        except KeyError as err:
            raise Exception("Required SCRUT_TESTS_DIR env var not set") from err

        try:
            scrut_user_provided_env = env.pop("SCRUT_USER_PROVIDED_ENV")
        except KeyError as err:
            raise Exception("Required SCRUT_USER_PROVIDED_ENV env var not set") from err

        # Expanded string macros (e.g. `$(location)`, `$(exe_target)`, etc.)
        # are absolute paths locally, but relative on RE. Try to make user
        # provided envs absolute so the tests can use them from any directory.
        #
        # NOTE: `buck run` scrut, because it's local, gets absolute paths.
        # So path resolution only matters here in unittestify.
        #
        # See https://fburl.com/workplace/tisvmbto
        if scrut_user_provided_env:
            # Unfortunate side-effect of needing to take a list of selects
            # to join into a string.
            assert (
                scrut_user_provided_env[-1] == ","
            ), f"missing expected trailing comma in `{scrut_user_provided_env}`"
            scrut_user_provided_env = scrut_user_provided_env[:-1]

            cwd = os.getcwd()
            for k in scrut_user_provided_env.split(","):
                try:
                    value = env[k]
                except KeyError as err:
                    raise Exception(
                        f"User provided env `{k}` is not in the environment",
                    ) from err

                # On Windows, buck will give you a command string rather than
                # an executable path when using `exe` or `exe_target`.
                # The string is the interpreter followed by the script path.
                # Note: Using `location` or pointing to a `command_alias`
                # with envs (or args) will give you an executable path.
                if (
                    os.name == "nt"
                    and (parts := value.partition(" "))
                    and (interpreter := parts[0])
                    and (script := parts[2])
                    and os.path.isabs(interpreter)
                    and interpreter.lower().endswith((".exe", ".bat", ".cmd"))
                    and not os.path.isabs(script)
                    and os.path.lexists(script)
                ):
                    env[k] = interpreter + " " + os.path.join(cwd, script)
                    continue

                # Do not resolve paths to avoid changing arg0's name.
                # Do not normalize paths to avoid incorrectly handling symlinks.
                if not os.path.isabs(value) and os.path.lexists(value):
                    env[k] = os.path.join(cwd, value)

        scrut_append_tests_dir = (
            env.pop("SCRUT_APPEND_TESTS_DIR")
            if "SCRUT_APPEND_TESTS_DIR" in env
            else None
        )

        scrut_prepend_tests_dir = (
            env.pop("SCRUT_PREPEND_TESTS_DIR")
            if "SCRUT_PREPEND_TESTS_DIR" in env
            else None
        )

        # TODO(T138035235) coverage is currently using wrong libs
        if "LD_PRELOAD" in env:
            env.pop("LD_PRELOAD")

        # Since prepend/append files are in another filegroup, keep track of common path
        base_path = Path(scrut_tests_dir).parent.parent

        # Add bootstrap/teardown for each test
        if scrut_append_tests_dir is not None:
            scrut_args.extend(
                [
                    "--append-test-file-paths={}".format(
                        testpath.relative_to(base_path)
                    )
                    for testpath in Path(scrut_append_tests_dir).iterdir()
                ],
            )
        if scrut_prepend_tests_dir is not None:
            scrut_args.extend(
                [
                    "--prepend-test-file-paths={}".format(
                        testpath.relative_to(base_path)
                    )
                    for testpath in Path(scrut_prepend_tests_dir).iterdir()
                ],
            )

        # Run test subcommand from scrut
        args = [os.path.join(os.getcwd(), scrut_location), "test"] + scrut_args

        method_names = {}
        for fmt, suffixes in {
            "cram": [".t", ".cram"],
            "markdown": [".md", ".markdown", ".scrut"],
        }.items():
            for suffix in suffixes:
                for testpath in Path(scrut_tests_dir).glob(f"**/*{suffix}"):
                    method_name = scruttest.generate_test_method_name(testpath)
                    if method_name in method_names:
                        raise Exception(
                            "Colliding test names: {} and {} both result in {}".format(
                                testpath,
                                method_names[method_name],
                                method_name,
                            )
                        )
                    method_names[method_name] = testpath

                    fmt_args = []
                    if fmt == "cram":
                        fmt_args.append("--cram-compat")
                        fmt_args.append("--combine-output")
                        fmt_args.append("--keep-output-crlf")

                    setattr(
                        cls,
                        method_name,
                        _make_scrut_test_method(
                            args=args
                            + fmt_args
                            + [str(testpath.relative_to(base_path))],
                            env=env,
                            cwd=Path.joinpath(Path(os.getcwd()), base_path),
                        ),
                    )


def _make_scrut_test_method(
    *, args: list[str], env: dict[str, str], cwd: Path
) -> Callable[[], None]:
    # pyre-fixme[2]: Parameter must be annotated.
    def runsingletest(self) -> None:
        try:
            subprocess.run(
                args,
                capture_output=True,
                cwd=cwd,
                check=True,
                env=env,
                text=True,
                encoding="utf-8",
            )
        except OSError as err:
            cmd = " ".join(args)
            raise self.fail(
                f"`scrut {cmd}` failed due to {err.__class__.__name__}:\n{err}"
            )
        except subprocess.CalledProcessError as err:
            sys.tracebacklimit = 0
            fail = scruttest.ScrutAssertionError(
                f"Scrut test failed {err}",
                cmd=err.cmd,
                output=err.output,
            )
            print("\n" + fail.render())
            if err.stderr:
                print("\n" + err.stderr, file=sys.stderr)
            raise fail from None

    return runsingletest


ScrutTests.auto_generate_tests()
