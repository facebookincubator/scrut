# Test Case

Each [test document](/docs/reference/fundamentals/test-document/) contains zero or more *test cases*.
Each test case should document and prove a single thing about the code under test (e.g. a single command line of the tested CLI). Test cases are the smallest unit of testing. They can fail or pass independently of each other. They are executed in the order they appear in the [test document](/docs/reference/fundamentals/test-document/).

## Anatomy

Independent of the format (Markdown, Cram) that a test case is written, it consists of the following components:

| Component | Required | Description |
| --- | --- | --- |
| **Title** | No | An optional title for the test case, so that a human can understand what the test case is intended to prove. |
| **Comment** | No | An optional comment leaving space for more description |
| **[Shell Expression](/docs/reference/fundamentals/shell-expression/)** | Yes | The subject of the test ("that what is being tested"). |
| **[Output Expectations](/docs/reference/fundamentals/output-expectations/)** | No | Any amount of assertions of the output that the [shell expression](/docs/reference/fundamentals/shell-expression/) will print |
| **[Exit Code](/docs/reference/behavior/exit-codes/)** | No | The expected exit code that the [shell expression](/docs/reference/fundamentals/shell-expression/) must end in. |
| **[Configuration](/docs/reference/fundamentals/inline-configuration/)** | No | Detailed, per-test-case configuration. |


## Format

Find more about the formatting of test cases in:
- [Reference > Formats > Markdown Format](/docs/reference/formats/markdown-format/#test-case-anatomy)
- [Reference > Formats > Cram Format](/docs/reference/formats/cram-format/)
