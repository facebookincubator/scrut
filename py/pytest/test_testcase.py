# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

import unittest

from clifoundation.scrut.py.pyscrut import TestCase


class TestCaseTestCase(unittest.TestCase):
    def test_create_testcase(self) -> None:
        testcase = TestCase(
            title="the title",
            shell_expression="the shell expression",
            exit_code=123,
            expectations=[
                "foo",
                "bar (glob)",
                "baz (regex*)",
            ],
            line_number=123,
            cram_compat=True,
        )
        self.assertEqual(testcase.title, "the title")
        self.assertEqual(testcase.shell_expression, "the shell expression")
        self.assertEqual(testcase.exit_code, 123)
        self.assertEqual(
            testcase.expectations,
            [
                ("equal", list(b"foo"), False, False),
                ("glob", list(b"bar"), False, False),
                ("regex", list(b"baz"), True, True),
            ],
        )
        self.assertEqual(testcase.line_number, 123)


if __name__ == "__main__":
    unittest.main()
