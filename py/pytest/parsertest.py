# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

from abc import abstractmethod
from typing import Any, List

from clifoundation.scrut.py.pyscrut import Output, TestCase


class ParserTestCase:
    @abstractmethod
    # pyre-fixme[2]: Parameter annotation cannot be `Any`.
    def assertEqual(self, first: Any, second: Any, msg: Any = ...) -> None:
        ...

    @abstractmethod
    def _testcases(self) -> List[TestCase]:
        ...

    @abstractmethod
    def _expect_line(self) -> int:
        ...

    def test_validate_valid_input(self) -> None:
        output = Output(b"Hello\n", b"", 0)
        ok, error = self._testcases()[0].validate(output, "the-location")
        self.assertEqual(True, ok)
        self.assertEqual(
            "Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped\n",
            error,
        )

    def test_fail_on_invalid_exit_code(self) -> None:
        result = Output(b"Hello\n", b"", 123)
        ok, error = self._testcases()[0].validate(result, "the-location")
        self.assertEqual(False, ok)
        self.assertEqual(
            """// =============================================================================
// @ the-location:{expected_line}
// -----------------------------------------------------------------------------
// # This is a test
// -----------------------------------------------------------------------------
// $ echo Hello
// =============================================================================

unexpected exit code
  expected: 0
  actual:   123

## STDOUT
#> Hello
## STDERR


Summary: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
""".format(
                expected_line=self._expect_line()
            ),
            error,
        )

    def test_validation_invalid_input(self) -> None:
        result = Output(b"Wrong", b"", 0)
        ok, error = self._testcases()[0].validate(result, "the-location")
        self.assertEqual(False, ok)
        print(ok)
        self.assertEqual(
            """// =============================================================================
// @ the-location:{expected_line}
// -----------------------------------------------------------------------------
// # This is a test
// -----------------------------------------------------------------------------
// $ echo Hello
// =============================================================================

   1  | - Hello
1     | + Wrong


Summary: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
""".format(
                expected_line=self._expect_line()
            ),
            error,
        )
