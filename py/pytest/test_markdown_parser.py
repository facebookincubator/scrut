# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

# pyre-strict

import unittest
from typing import List

from clifoundation.scrut.py.pyscrut import MarkdownParser, TestCase

from . import parsertest


class TestMarkdownParser(unittest.TestCase, parsertest.ParserTestCase):
    def test_extract_testcases(self) -> None:
        testcases = self._testcases()
        self.assertEqual(1, len(testcases))
        self.assertEqual("This is a test", testcases[0].title)
        self.assertEqual("echo Hello", testcases[0].shell_expression)
        self.assertEqual(1, len(testcases[0].expectations))
        self.assertEqual(
            ("equal", list(b"Hello"), False, False), testcases[0].expectations[0]
        )

    def _testcases(self) -> List[TestCase]:
        parser = MarkdownParser(["scrut"])
        _, testcases = parser.parse(
            "This is a test\n```scrut\n$ echo Hello\nHello\n```"
        )
        return testcases

    def _expect_line(self) -> int:
        return 3


if __name__ == "__main__":
    unittest.main()
