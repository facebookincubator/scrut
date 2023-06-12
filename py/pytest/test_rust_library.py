import unittest


class TestRustLibraryLoader(unittest.TestCase):
    def test_use_library(self) -> None:
        from clifoundation.scrut.py import pyscrut

        self.assertTrue(True, "pyscrupt librar loaded")
        # self.assertEqual("Hello World", pyscrut.hello_word())


if __name__ == "__main__":
    unittest.main()
