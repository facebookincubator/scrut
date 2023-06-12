import unittest

from clifoundation.scrut.py.pyscrut import Output


class TestOutput(unittest.TestCase):
    def test_create_output_int(self) -> None:
        output = Output(b"the stdout", b"the stderr", 123)
        self.assertEqual(b"the stdout", bytearray(output.stdout))
        self.assertEqual(b"the stderr", bytearray(output.stderr))
        self.assertEqual("123", output.exit_code)

    def test_create_output_string(self) -> None:
        output = Output(b"the stdout", b"the stderr", "123")
        self.assertEqual(b"the stdout", bytearray(output.stdout))
        self.assertEqual(b"the stderr", bytearray(output.stderr))
        self.assertEqual("123", output.exit_code)

    def test_create_output_timeout(self) -> None:
        output = Output(b"the stdout", b"the stderr", "timeout[123ms]")
        self.assertEqual(b"the stdout", bytearray(output.stdout))
        self.assertEqual(b"the stderr", bytearray(output.stderr))
        self.assertEqual("timeout[123ms]", output.exit_code)

    def test_create_output_unknown(self) -> None:
        output = Output(b"the stdout", b"the stderr", "whatever")
        self.assertEqual(b"the stdout", bytearray(output.stdout))
        self.assertEqual(b"the stderr", bytearray(output.stderr))
        self.assertEqual("unknown", output.exit_code)


if __name__ == "__main__":
    unittest.main()
