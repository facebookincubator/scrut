# Exit Codes

The expected exit code of a [shell expression](/docs/reference/fundamentals/shell-expression/) in a [test case](/docs/reference/fundamentals/test-case/) can be denoted with a integer in square brackets. For example:

````markdown title="example.md" showLineNumbers {6}
# The command is expected to end with exit code 2

```scrut
$ some-command --foo
an expected line of output
[2]
```
````

Unless otherwise specified an exit code of `0` (zero) is expected. You can explicitly denote it with `[0]` if you prefer.

:::note

Exit code evaluation happens before [output expectations](/docs/reference/fundamentals/output-expectations/) are evaluated. That means if the exit code fails then no output validation is attempted.

:::

## Skip Tests with Exit Code 80

If any [test case](/docs/reference/fundamentals/test-case/) in a test file exist with exit code `80`, then all [test case](/docs/reference/fundamentals/test-case/) in that file are skipped.

This is especially helpful for OS specific tests etc. Imagine:

````markdown title="example.md" showLineNumbers {4}
Run tests in this file only on Mac

```scrut
$ [[ "$(uname)" == "Darwin" ]] || exit 80
```
````

:::note

The exit code can be configured with the [`skip_document_code` configuration directive](/docs/reference/fundamentals/inline-configuration/).

:::

## Scrut Exit Code

Scrut itself communicates the outcome of executions with exit codes. Currently three possible exit codes are used:

- `0`: Command succeeded, all is good (`scrut test`, `scrut create`, `scrut update`)
- `1`: Command failed with error (`scrut test`, `scrut create`, `scrut update`)
- `50`: Validation failed (`scrut test` only)
