# Cram Format

:::warning

For new tests, prefer using the [Markdown format](/docs/reference/formats/markdown-format/) which was introduced with two goals in mind:

1. **Tests ❤️ Documentation**: The value of tests is not only in proving behavior, but also in documenting it - and thereby also in teaching it. The Markdown format allows you to keep tests around in a way that future generations of maintainers will love you for.
2. **Bad Spaces 👾**: To denote an expected empty line of output in Cram format you have to provide two empty spaces `  `. This goes counter a lot of default behavior in the development toolchain. Many CI/CD tools are tuned to automatically ignore changes that only affect white spaces. Code review tools often deliberately hide white spae changes. White spaces are generally hard to see in code editors - if they are visualized at all. Breaking tests that are caused by an accidentally removed or added space cause rage quitting.

See also additional differences in the [way tests are executed](/docs/reference/behavior/execution-model/).

:::


The Cram document format is supported for legacy reasons. The general guidance to write [test cases](/docs/reference/fundamentals/test-case/) in Cram [test documents](/docs/reference/fundamentals/test-document/) is:

- The first line of a [shell expression](/docs/reference/fundamentals/shell-expression/) must start with `  $ ` (space + space + dollar + space), any subsequent with `  > ` (space + space + closing angle bracket + space).
  - This is different from Markdown Scrut syntax. Be mindful of the additional spaces.
- Lines following the [shell expression](/docs/reference/fundamentals/shell-expression/), that are also indented with two spaces, are considered [output expectations](/docs/reference/fundamentals/output-expectations/)
  - If an [exit code](/docs/reference/behavior/exit-codes/) other than `0` is expected, it can be denoted in square brackets `[123]` once per [test case](/docs/reference/fundamentals/test-case/)
  - Note: Empty output lines (=empty *shell expectations*) must still have two leading space characters
  - Note: A fully empty line (no leading spaces) denotes the end of the current [test case](/docs/reference/fundamentals/test-case/)
- If the [shell expression](/docs/reference/fundamentals/shell-expression/) is preceded by a non-empty line (that is *not* indented) the line is considered the *title* of the [test case](/docs/reference/fundamentals/test-case/)

Here an example:

```cram
This is a comment
  $ scrut --help
  Scrut help output

Another test case in the same document
  $ scrut --version
  Scrut version output
```

Multiple tests test cases can be written in sequence without any empty lines in between:

```cram
A title for the first test case
  $ first --command
  $ second --command
  $ third --command
  Output Expectation
```
