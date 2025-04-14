# Test Document

*Test documents* are the files that contain the instructions of *what* ought to be tested and *how* to test it. They are the main entry point for working with `scrut` from the command line. All test documents contain zero or more [test cases](/docs/reference/fundamentals/test-case/).

## Document Formats

Scrut supports two formats for test documents:
- [Markdown](/docs/reference/formats/markdown-format/), the default and recommended format for writing test documents.
- [Cram](/docs/reference/formats/markdown-format/), supported for legacy reasons to run or migrate tests written for the now [deprecated Cram framework](https://github.com/aiiie/cram)

## Document Writing Recommendations

Consider test documents not only as stashes for test cases, but also as documentation for the tested functionality. Maintaining CLIs, as any software, long term is a challenge. Using the Markdown format Scrut provides an opportunity to store knowledge about systems (i.e. behavior of the CLI) together with validation of that knowledge (the test cases).

## File Structure Recommendation

There are two common patterns to structure test documents in Scrut:

- **Coherent Test Suite** (recommended): One test file represents one use-case or behavior. This makes it easy to identify broken functionality.
- **List of Tests**: One test file contains a list of simple, not necessarily related tests. This makes it easy to cover a lot of functionality quickly, but at the price of harder maintainability down the line.
