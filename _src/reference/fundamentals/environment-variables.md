# Environment Variables

Scrut sets some environment variables when executing tests. They are set just before executing individual [test cases](/docs/reference/fundamentals/test-case/) in a test document. While they can be overwritten within a [test case](/docs/reference/fundamentals/test-case/), they are set anew for each [test case](/docs/reference/fundamentals/test-case/).

## Scrut specific environment variables

- `TESTDIR`: absolute path of the directory where the document that contains the test that is currently being executed is in
- `TESTFILE`: name of the [test document](/docs/reference/fundamentals/test-document/) that contains the test that is currently being executed
- `TESTSHELL`: shell that in which the test is being executed in (default `/bin/bash`, see `--shell` flag on commands)
- `TMPDIR`: absolute path to a temporary directory that will be cleaned up after the test is executed. This directory is shared in between all executed tests across all test documents. Tools like `mktemp` will make use of `TMPDIR` automatically.
- `SCRUT_TEST`: path to the test document and the line number, separated by a colon (e.g. `some/test.md:123`). *This variable is recommend to use when deciding whether an execution is within Scrut.*

:::tip

Use the `SCRUT_TEST` variable to decide whether an execution is within Scrut. This is useful when you want to be aware of that fact from within your CLI during test execution.

:::

## Common (linux) environment variables

Scrut sets the following environment variables to their default values:

- `CDPATH`: empty
- `COLUMNS`: `80`
- `GREP_OPTIONS`: empty
- `LANG`: `C`
- `LANGUAGE`: `C`
- `LC_ALL`: `C`
- `SHELL`: Same as `TESTSHELL`, see above
- `TZ`: `GMT`

## (Optional) Cram environment variables

When using the `--cram-compat` flag, or when a Cram `.t` test document is being executed, the following additional environment variables will be exposed for compatibility:

- `CRAMTMP`: if no specific working directory was provided (default), then it contains the absolute path to the temporary directory in which per-test-file directories will be created in which those test files are then executed in (`CRAMTMP=$(realpath "$(pwd)/..")`); otherwise the path to the provided working directory
- `TMP`: same as `TMPDIR`
- `TEMP`: same as `TMPDIR`
