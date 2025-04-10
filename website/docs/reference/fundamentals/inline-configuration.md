# Inline Configuration

Scrut support two kinds of inline configuration syntax:

1. **Per Test Document** (document-wide) configuration, which can be defined at the start of the [test document](/docs/reference/fundamentals/test-document/)
2. **Per Test Case** (test-case-wide) configuration, which can be defined with each individual [test case](/docs/reference/fundamentals/test-case/)

:::warning

This configuration method is only supported for test documents using the [Markdown format](/docs/reference/formats/markdown-format/). There is no equivalent in the  [Cram format](/docs/reference/formats/cram-format/).

:::

## Example

````markdown title="example.md" showLineNumbers
---
# optional document-wide YAML configuration
total_timeout: 30s
---

# The test document

The initial block that is initialized with `---` and terminated with `---` contains
the configuration in YAML notation.

## A simple test

```scrut
$ echo Hello One
Hello One
```

The above test does not contain any per-test configuration

## A test with configuration

```scrut {timeout: 10s}
$ echo Hello Two
Hello Two
```

The above test contains per-test configuration
````

Some inline-configuration attribute can overwritten by parameters provided on the command-line (see below). The order of precedence is:
1. Command-line parameter
2. Per Test Case configuration
3. Per Test Document configuration
4. Default

## Test Document Configuration

| Name            | Type                                                           | Corresponding Command Line Parameter | Description                                                                                                                                                                                                                                              |
| --------------- | -------------------------------------------------------------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `append`        | list of paths to documents                                     | `--append-test-file-paths`           | Include these paths in order, as if they were part of this document. All tests within the appended paths are appended to the tests defined in this document. Use-case is common/shared test tear-down. Paths must be relative to the current `$TESTDIR`. |
| `defaults`      | See below [Test Case Configuration](#test-case-configuration)   | n/a                                  | Defaults for per-test-case configuration within the test file.                                                                                                                                                                                           |
| `prepend`       | list of paths to documents                                     | `--prepend-test-file-paths`          | Include these paths in order, as if they were part of this document. All tests within the prepend paths are prepended to the tests defined in this document. Use-case is common/shared test setup. Paths must be relative to the current `$TESTDIR`.     |
| `shell`         | string                                                         | `--shell`                            | The path to the shell. If a full path is not provided, then the command must be in `$PATH`. **Only `bash` compatible shells are currently supported!**                                                                                                   |
| `total_timeout` | [duration string](https://docs.rs/humantime/latest/humantime/) | `--timeout-seconds`                  | All tests within the document (including appended and prepended) must finish executing within this time.                                                                                                                                                 |

**Defaults (Markdown and Cram)**

```yaml
append: []
defaults: {}
prepend: []
shell: bash
total_timeout: 15m
```

:::note

Per-document configuration in documents that are appended or prepended is ignored

:::

## Test Case Configuration

| Name                 | Type                                                                                                                | Corresponding Command Line Parameter         | Description                                                                                                                                                                                                                                                                                                                              |
| -------------------- | ------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `detached`           | boolean                                                                                                             | n/a                                          | Tell Scrut that the [shell expression](/docs/reference/fundamentals/shell-expression/) of this test will detach itself, so Scrut will not consider this a test (i.e. no output or exit code evaluation). Purpose is to allow the user to detach a command (like `nohup some-command &`) that is doing something asynchronous (e.g. starting a server to which the tested CLI is a client). |
| `environment`        | object                                                                                                              | n/a                                          | A set of environment variable names and values that will be explicitly set for the test.                                                                                                                                                                                                                                                 |
| `keep_crlf`          | boolean                                                                                                             | `--keep-output-crlf`                         | Whether CRLF should be translated to LF (=false) or whether CR needs to be explicitly handled (=true).                                                                                                                                                                                                                                   |
| `output_stream`      | enum (`stdout`, `stderr`, `combined`)                                                                               | `--combine-output` and `--no-combine-output` | Which output stream to choose when applying [output expectations](/docs/reference/fundamentals/output-expectations/): `stdout` (all expectations apply to what is printed on STDOUT), `stderr` (all expectations apply to what is printed on STDERR), `combined` (STDOUT and STDERR will combined into a single stream where all expectations are applied on)                                 |
| `skip_document_code` | positive integer                                                                                                    | n/a                                          | The exit code, that if returned by any test, leads to skipping of the whole document.                                                                                                                                                                                                                                                    |
| `timeout`            | null or [duration string](https://docs.rs/humantime/latest/humantime/)                                              | n/a                                          | A max execution time a test can run before it is considered failed (and will be aborted).                                                                                                                                                                                                                                                |
| `wait`               | null or [duration string](https://docs.rs/humantime/latest/humantime/) or [Wait Configuration](#wait-configuration) | n/a                                          | See [Wait Configuration](#wait-configuration)                                                                                                                                                                                                                                                                                            |

**Defaults (Markdown)**

```yaml
detached: false
environment: {}
keep_crlf: false
output_stream: stdout
skip_document_code: 80
timeout: null
wait: null
```

**Defaults (Cram)**

```yaml
detached: false
environment: {}
keep_crlf: true
output_stream: combined
skip_document_code: 80
timeout: null
wait: null
```

### Wait Configuration

This configuration corresponds to the per-test-case `detached` configuration and helps to write client / server tests where first a server is started (i.e. a test that runs detached) and then a client communicates with the server (i.e. a test that waits)

| Name      | Type                                                           | Description                                                                              |
| --------- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `timeout` | [duration string](https://docs.rs/humantime/latest/humantime/) | How long to wait for the test to run.                                                    |
| `path`    | null or string                                                 | If set then the wait will end early once the path exists. This path must be in `$TMPDIR` |


**Example**

````markdown
# A server/client test example

Show-case how a server/client test that initially starts a server

## Start a server

```scrut {detached: true}
$ my-server --start && touch "$TMPDIR"/server-started
```

## Run client test once server is up

```scrut {wait: {timeout: 5m, path: server-started}}
$ my-client --do-a-thing
```
````

:::note

The path provided with the `path` directive must be relative to `$TMPDIR`.
