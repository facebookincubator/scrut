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

All configuration that can be applied *per test document*.

### `append`

- Type: **list of paths to documents**
- Command Line Parameter: **`--append-test-file-paths`**
- Default: **`[]`**

The `append` configuration allows you to specify a list of document paths that should be included as if they were part of the current test document. All tests within the appended paths are executed after the tests defined in the current document. This is particularly useful for including common or shared test tear-down procedures. The paths specified must be relative to the current `$TESTDIR`.

**Example:**

```yaml
append:
  - "common-teardown.md"
  - "additional-tests.md"
```

:::warning

Per-document configuration, including defaults, from appended documents is ignored.

:::

### `defaults`

- Type: **object**
- Command Line Parameter: **n/a**
- Default: **`{}`**

The `defaults` configuration allows you to specify default values for per-test-case configurations within the test document. These defaults are applied to each test case unless overridden by specific configurations within the test case itself. This is useful for setting common configurations that apply to multiple test cases, reducing redundancy and ensuring consistency across tests.

**Example:**

```yaml
defaults:
  timeout: "5s"
  environment:
    FOO: "bar"
```

In the above example, each test case will have a default timeout of 5 seconds and an environment variable `FOO` set to "bar", unless these are explicitly overridden in the test case configuration.


### `prepend`

- Type: **list of paths to documents**
- Command Line Parameter: **`--prepend-test-file-paths`**
- Default: **`[]`**

The `prepend` configuration allows you to specify a list of document paths that should be included as if they were part of the current test document. All tests within the prepended paths are executed before the tests defined in the current document. This is particularly useful for including common or shared test setup procedures. The paths specified must be relative to the current `$TESTDIR`.

**Example:**

```yaml
prepend:
  - "common-setup.md"
  - "initial-tests.md"
```

:::warning

Per-document configuration, including defaults, from prepended documents is ignored.

:::

### `shell`

- Type: **string**
- Command Line Parameter: **`--shell`**
- Default (Linux, MacOS): **`/bin/bash`**
- Default (Windows): **`bash`**

The `shell` configuration specifies the path to the shell that should be used to execute the test cases. If a full path is not provided, the shell command must be available in the system's `$PATH`. Currently, only `bash` compatible shells are supported. This configuration is useful when you need to run tests in a specific shell environment that might have different features or behaviors compared to the default shell.

**Example:**

```yaml
shell: /bin/my-bash
```

:::tip

You can also overwrite the default shell using the `SCRUT_DEFAULT_SHELL` [environment variable](/docs/reference/fundamentals/environment-variables/).

:::

### `total_timeout`

- Type: **[duration string](https://docs.rs/humantime/latest/humantime/)**
- Command Line Parameter: **`--timeout-seconds`**
- Default: **`15m`**

The `total_timeout` configuration specifies the maximum duration allowed for all tests within the document to complete execution. This includes tests from both appended and prepended documents. If the total execution time exceeds this limit, the test run is aborted. This setting is useful for ensuring that test suites do not run indefinitely and helps in managing overall test execution time.

**Example:**

```yaml
total_timeout: "30m"
```

## Test Case Configuration

All configuration that can be applied *per test case* in Markdown test documents.

:::note

Mind that [Cram](/docs/reference/formats/cram-format/) does not support per-test-case configuration and that defaults for [Markdown](/docs/reference/formats/markdown-format/) and Cram have slightly different default values. If they differ then *Markdown Default* and *Cram Default* are provided below, if they are the same then only *Default* is mentioned.

:::

### `detached`

- Type: **boolean**
- Command Line Parameter: **n/a**
- Default: **`false`**

Tell Scrut that the [shell expression](/docs/reference/fundamentals/shell-expression/) of this test will detach itself, so Scrut will not consider this a test (i.e. no output or exit code evaluation). Purpose is to run detached commands (like `nohup some-command &`) that are doing something asynchronous (e.g. starting a server to which the tested CLI is a client).

**Example:**

````markdown showLineNumbers
```scrut {detached: true}
$ my-server --start
```
````

### `detached_kill_signal`

- Type: **enum(`disabled`, `SIGINT`, `int`, 2, `SIGABRT`, `abrt`, 6, ...)**, see [here](https://docs.rs/nix/0.29.0/nix/sys/signal/enum.Signal.html#variants) for all supported names
- Command Line Parameter: **n/a**
- Default: **`term`**

If `detached` is set to `true` then this configuration specifies the signal that is send to the detached process when all testcases in the test document have been executed.

**Example:**

````markdown showLineNumbers
```scrut {detached: true, detached_kill_signal: term}
$ my-server --start
```
````

:::warning

Kill signals are only supported on Linux and MacOS. They are ignored (but validated) on Windows.

:::

### `fail_fast`

- Type: **boolean**
- Command Line Parameter: **n/a**
- Default: **`false`**

If set to `true`, stops execution of the entire test document immediately if this test case fails for any reason (exit status, snapshot validation, etc.). All remaining test cases in the document will be marked as skipped. This is useful when a critical test fails and subsequent tests are not meaningful or would fail anyway.

**Example:**

````markdown showLineNumbers
```scrut {fail_fast: true}
$ critical-setup-command
```
````

In this example, if `critical-setup-command` fails, all subsequent tests in the document are skipped.

### `environment`

- Type: **object**
- Command Line Parameter: **n/a**
- Default: **`{}`**

This configuration allows you to set environment variables for the test case. The environment variables are specified as key-value pairs in an object. These variables are set in the environment where the test case is executed.

**Example:**

````markdown
```scrut {environment: {"FOO": "bar"}}
$ echo $FOO
bar
```
````

### `keep_crlf`

- Type: **boolean**
- Command Line Parameter: **`--keep-output-crlf`**
- Markdown Default: **`false`**
- Cram Default: **`true`**

This configuration determines whether carriage return and line feed (CRLF) sequences should be preserved in the output. When set to `true`, CRLF sequences are kept as-is, which is useful for tests that require exact output matching, including line endings. When set to `false`, CRLF sequences are translated to line feed (LF) only, which is the default behavior and is typically used for compatibility with Unix-style line endings.

**Example:**

````markdown showLineNumbers
```scrut {keep_crlf: 42}
$ echo -e "Give CRLF\r\n"
Give CRLF\r (escape)
```
````

### `output_stream`

- Type: **enum(`stdout`, `stderr`, `combined`)**
- Command Line Parameter: **`--combine-output`** and **`--no-combine-output`**
- Markdown Default: **`stdout`**
- Cram Default: **`combined`**

This configuration specifies which output stream to use when applying [output expectations](/docs/reference/fundamentals/output-expectations/). The options are:
  - `stdout`: All expectations apply to what is printed on STDOUT.
  - `stderr`: All expectations apply to what is printed on STDERR.
  - `combined`: STDOUT and STDERR are combined into a single stream where all expectations are applied.

**Example:**

````markdown showLineNumbers
```scrut {output_stream: combined}
$ echo "This goes to STDERR" >&2 && echo "This goes to STDOUT"
This goes to STDERR
This goes to STDOUT
```
````

### `skip_document_code`

- Type: **positive integer**
- Command Line Parameter: **n/a**
- Default: **`80`**

This configuration specifies an exit code that, if returned by any test within the document, will cause the entire document to be skipped. This is useful for scenarios where a specific condition, indicated by the exit code, should prevent further tests from running. The default value is `80`.

**Example:**

````markdown showLineNumbers
```scrut {skip_document_code: 42}
$ echo "I give up" && exit 42
```
````

### `strip_ansi_escaping`

- Type: **boolean**
- Command Line Parameter: **n/a**
- Default: **`false`**

This configuration determines whether ANSI escape sequences should be stripped from the CLI output before validation. When set to `true`, all ANSI escape sequences are removed, which is useful for tests that require output without formatting codes. When set to `false`, ANSI escape sequences are preserved, allowing for validation of formatted output.

**Example:**

````markdown title="example.md" showLineNumbers
```scrut {strip_ansi_escaping: true}
$ echo -e "\033[31mThis is red text\033[0m"
This is red text
```
````

### `timeout`

- Type: **[duration string](https://docs.rs/humantime/latest/humantime/)**
- Command Line Parameter: **n/a**
- Default: unset

The `timeout` configuration specifies the maximum duration allowed for a single test case to complete execution. If the test case does not finish within this time frame, it is aborted and the execution is considered an error. This setting is useful for ensuring that individual tests do not run indefinitely and helps in managing the execution time of each test case.

````markdown showLineNumbers
```scrut {timeout: 5s}
$ sleep 10
```
````

### `wait`

- Type: **[duration string](https://docs.rs/humantime/latest/humantime/)**, or **`{wait: {timeout: <duration-string>, path: <path>}}`**
- Command Line Parameter: **n/a**
- Default: unset

This configuration is used to specify a waiting period for a test case, which is particularly useful in scenarios where a test needs to wait for a certain condition to be met before proceeding. The `wait` configuration can be set to a duration string to specify a simple wait time, or it can be a more complex configuration that includes both a timeout and a path condition. If the `path` is specified, the wait will end early if the specified path exists, allowing for synchronization with external processes or conditions.

**Example (simple, only timeout):**

````markdown showLineNumbers
```scrut {wait: "10s"}
$ echo "Waiting for 10 seconds"
```
````

**Example (extended, timeout and path):**

````markdown showLineNumbers
# Run something that creates `sock` file

```scrut {detached: true}
$ start-something --sock-file "$TMPDIR/sock"
```

The above executes `start-something` that creates the file `$TMPDIR/sock` when it is ready.

# Wait at most 10 seconds or until `sock` file exists

```scrut {wait: {timeout: "10s", path: "sock"}}
$ echo "Can work with $TMPDIR/sock now"
```

The above waits for `$TMPDIR/sock` to exist for at most 10 seconds.
````
