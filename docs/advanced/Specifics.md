# Specifics

This chapter describes behaviors of Scrut that should be known by the user to prevent surprises in the wrong moment.

## Test output

Executing a test with Scrut results either in success (when all expectations in the test match) or failure (when at least one expectation in the test does not match).

Scrut supports multiple *output renderers*, which yield a different representation of the test results.

### Pretty Renderer (default)

Scrut will always tell you what it did:

```bash
$ scrut test selftest/cases/regex.md
Summary: 1 file(s) with 8 test(s): 8 succeeded, 0 failed and 0 skipped
```

In case of failure the `pretty` default renderer will provide a human-readable output that points you to the problem with the output:

```bash
$ scrut test a-failing-test.md
// =============================================================================
// @ /path/to/a-failing-test.md:10
// -----------------------------------------------------------------------------
// # One conjunct expression
// -----------------------------------------------------------------------------
// $ echo Foo && \
//   echo Bar
// =============================================================================

1  1  |   Foo
   2  | - BAR
2     | + Bar
3     | + Baz
```

The failure output consists of two components:

1. The failure header, which consists of all initial lines that start with `//`, indicates the position
2. The failure body, which consists of all the following lines, indicates the problem

**Header**

The header contains three relevant information. Given the above output:

- `@ /path/to/a-failing-test.md:4`, tells you that the test that failed is in the provided file `/path/to/a-failing-test.md` and that the shell expression (that failed the test) starts in line four of that file.
- `# <test title>`, gives you the optional title of the test in the file. See [File Formats](File_Formats.md)) to learn more. *If the test does not have a title, this line is omitted.*
- `$ <test command>`, is the shell expectation from the test file that is tested and that has failed. Again, see [File Formats](File_Formats.md)) for more information.

**Body**

There are two possible variants that the `diff` renderer may return:

1. Failed [output expectations](Expectations.md)
2. Failed [exit code expectation](#exit-codes)

The above output is a failed output expectations and you can read it as following:

- `1  1  |   Foo`: This line was printed as expected. The left hand `1` is the number of the output line and the right hand `1` is the number of the expectation.
- `   2  | - BAR`: This line was expected, but not printed. The left hand omitted number indicates that it was not found in output. The right hand number tells that this is the second expectation. The `-` before the line `Bar` emphasizes that this is a missed expectation.
- `2     | + Bar`: This line was printed and expected. The left hand `2` is the number of the output line and the right hand `3` is the number of the expectation.
- `3     | + Baz`: This line was printed unexpectedly. The left hand `3` is the number of the output line the omitted right hand number implies there is no expectation that covers it. The `+` before the line `Zoing` emphasizes that this is a "surplus" line.

> **Note**: If you work with test files that contain a large amount of tests, then you may want to use the `--absolute-line-numbers` flag on the command line: instead of printing the relative line number for each test, as described above, it prints absolute line numbers from within the test file. Assuming the `Foo` expectation from above is in line 10 of a file, it would read `13  13  |   Foo` - and all subsequent output liens with respective aligned line numbers.

An example for the body of an *exit code expectation*:

```
unexpected exit code
  expected: 2
  actual:   0

## STDOUT
#> Foo
## STDERR
```

This should be mostly self-explanatory. Scrut does not provide any output expectation failures, because it assumes that when the exit code is different, then it is highly likely that the output is very different - and even if not, it would not matter, as it failed anyway.

The tailing `## STDOUT` and `## STDERR` contain the output lines (prefixed with `#> `) that were printed out from the failed execution.

### Diff renderer

The `diff` renderer, that can be enabled with `--renderer diff` (or `-r diff`), prints a diff in the [unified format](https://en.wikipedia.org/wiki/Diff#Unified_format).

```bash
$ scrut test -r diff a-failing-test.md
--- /path/to/a-failing-test.md
+++ /path/to/a-failing-test.md.new
@@ -14 +14,2 @@ malformed output: One conjunct expression
-BAR
+Bar
+Baz
```

> **Note**: The created diff is compatible with the `patch` command line tool (e.g. `patch -p0 < <(scrut test -r diff a-failing-test.md)`).

### JSON and YAML renderer

These renderer are primarily intended for automation and are to be considererd experimental.
You can explore them using `--renderer yaml` or respective `--renderer json`.

## Test environment variables

Scrut sets a list of environment variables for the execution. These are set _in addition to and overwriting_ any environment variables that are set when `scrut` is being executed.

> **Note**: If you need an empty environment, consider executing using [`env`](https://man7.org/linux/man-pages/man1/env.1.html), like `env -i scrut test ..` instead

### Scrut specific environment variables

- `TESTDIR`: contains the absolute path of the directory where the file that contains the test that is currently being executed is in
- `TESTFILE`: contains the name of the file that contains the test that is currently being executed
- `TESTSHELL`: contains the shell that in which the test is being executed in (default `/bin/bash`, see `--shell` flag on commands)
- `TMPDIR`: contains the absolute path to a temporary directory that will be cleaned up after the test is executed. This directory is shared in between all executed tests across all test files.

### Common (linux) environment variables

- `CDPATH`: empty
- `COLUMNS`: `80`
- `GREP_OPTIONS`: empty
- `LANG`: `C`
- `LANGUAGE`: `C`
- `LC_ALL`: `C`
- `SHELL`: Same as `TESTSHELL`, see above
- `TZ`: `GMT`

### (Optional) Cram environment variables

When using the `--cram-compat` flag, or when a Cram `.t` test file is being executed, the following additional environment variables will be exposed for compatibility:

- `CRAMTMP`: if no specific work directory was provided (default), then it contains the absolute path to the temporary directory in which per-test-file directories will be created in which those test files are then executed in (`CRAMTMP=$(realpath "$(pwd)/..")`); otherwise the path to the provided work directory
- `TMP`: same as `TMPDIR`
- `TEMP`: same as `TMPDIR`

## Test work directory

By default `scrut` executes all tests in a dedicated directory _per test file_. This means _all tests within one file are being executed in the same directory_. The directory is created within the system temporary directory. It will be removed (including all the files or directories that the tests may have created) after all tests in the file are executed - or if the execution of the file fails for any reason.

This means something like the following can be safely done and will be cleaned up by Scrut after the test finished (however it finishes):

````markdown
# Some test that creates a file

```scrut
$ date > file
```

The `file` lives in the current directory

```scrut
$ test -f "$(pwd)/file"
```
````

The directory within which tests are being executed can be explicitly set using the `--work-directory` parameter for the `test` and `update` commands. If that parameter is set then _all tests_ from _all test files_ are executed run within that directory, and the directory is _not removed_ afterwards.

> **Note**: In addition to the work directory Scrut also creates and cleans up a temporary directory, that is accessible via `$TMPDIR`. Tools like `mktemp` automatically use it (from said environment variable).

## Test execution

As Scrut is primarily intended as an integration testing framework for CLI applications, it is tightly integrated with the shell.
Each Scrut test must define a [shell expression](File_Formats.md#test-case-anatomy) (called an "execution").
Each of those executions is then run within an actual shell (bash) process, as they would be when a human or automation would execute the expression manually on the shell.

With that in mind:

- Each execution from the same test file is executed in an individual shell process.
  - Scrut currently only supports `bash` as shell process.
  - Each subsequent execution within the same file inherits the state of the previous execution: environment variables, shell variables, functions, settings (`set` and `shopt`).
- Tests within the same file are executed in sequential order.
- Executions happen in a [temporary work directory](#test-work-directory), that is initially empty and will be cleaned up after the last executions of the test file has run (or when executions are [skipped](#skip-tests-with-exit-cod)).
- Executions may be detached, but Scrut will not clean up (kill) or wait for detached child processes

### Execution within a custom shell

While Scrut currently only supports `bash` (>= 3.2) a custom shell can be provided with the `--shell` command line parameter.
To understand how that works consider the following:

```bash
$ echo "echo Hello" | /bin/bash -
Hello
```

What the above does is piping the string `echo Hello` into the `STDIN` of the process that was started with `/bin/bash -`.
Scrut pretty much does the same with each shell expressions within a test file.

So why provide a custom `--shell` then?
This becomes useful in two scenarios:
1. You need to execute the same code before Scrut runs each individual expression
2. You need Scrut to execute each expression in some isolated environment

For (1) consider the following code:

```bash
#!/bin/bash

# do something in this wrapper script
source /my/custom/setup.sh
run_my_custom_setup

# consume and run STDIN
source /dev/stdin
```

For (2) consider the following:

```bash
#!/bin/bash

# do something in this wrapper script
source /my/custom/setup.sh
run_my_custom_setup

# end in a bash process that will receive STDIN
exec ssh username@acme.tld /bin/bash
```

Instead of SSHing into a machine, consider also running a bash process in docker container.

## STDOUT and STDERR

Commands-line applications can generate output on to two streams: `STDOUT` and `STDERR`. There is no general agreement on which stream is supposed to contain what kind of data, but commonly `STDOUT` contains the primary output and `STDERR` contains logs, debug messages, etc. This is also the recommendation of the [CLI guidelines](https://clig.dev/#:~:text=primary%20output%20for%20your%20command).

Scrut validates CLI output via [Expectations](Expectations.md). Which output that entails can be configured via the [`output_stream` configuration directive](File_Formats.md#testcase-configuration) (and the `--(no-)combine-output` command-line parameters).

**Note:** While you can configure which output streams Scrut considers when evaluating output expecations, you can also steer this by using stream control bash primitives like `some-command 2>&1`.

## Exit Codes

You can denote the expected exit code of a shell expression in a testcase. For example:

````
The command is expected to end with exit code 2

```scrut
$ some-command --foo
an expected line of output
[2]
```
````

Unless otherwise specified an exit code of 0 (zero) is assumed. You can explicitly denote it with `[0]` if you prefer.

> **Note**: Exit code evaluation happens before output expectations are evaluated.

### Skip Tests with Exit Code 80

If any testcase in a test file exist with exit code `80`, then all testcases in that file are skipped.

This is especially helpful for OS specific tests etc. Imagine:

````
Run tests in this file only on Mac

```scrut
$ [[ "$(uname)" == "Darwin" ]] || exit 80
```
````

**Note:** The code that Scrut accepts to skip a whole file can be modified with the [`skip_document_code` configuration directive](File_Formats.md#testcase-configuration).

### Scrut Exit Code

Scrut itself communicates the outcome of executions with exit codes. Currently three possible exit codes are supported:

- `0`: Command succeeded, all is good (`scrut test`, `scrut create`, `scrut update`)
- `1`: Command failed with error (`scrut test`, `scrut create`, `scrut update`)
- `50`: Validation failed (`scrut test` only)

## Newline handling

[Newline](https://en.wikipedia.org/wiki/Newline) endings is a sad story in computer history. In Unix / MacOS ( / \*BSD / Amiga / ..) the standard line ending is the line feed (LF) character `\n`. Windows (also Palm OS and OS/2?) infamously attempted to make a combination of carriage return (CR) and line feed the standard: CRLF (`\r\n`). Everybody got mad and still is.

See the [`keep_crlf` configuration directive](File_Formats.md#testcase-configuration) to understand how Scrut handles LF and CRLF and how you can modify the default behavior.

## Execution Environment

A [Scrut test file](File_Formats.md) can contain arbitrary amounts of tests. Scrut provides a shared execution environment for all tests within a single file, which comes with certain behaviors and side-effects that should be known:

- **Shared Shell Environment**: Each subsequent testcase in the same file inherits the shell environment of the previous testcase. This means: All environment variables, shell variables, aliases, functions, etc that have are set in test are available to the immediate following test.
  - *Exception*: Environments from [`detached`](File_Formats.md#testcase-configuration) testcases are not passed along
- **Shared Ephemeral Directories**: Each testcase in the same test file executes in the the same work directory and is provided with the same temporary directory ([`$TEMPDIR`](#test-environment-variables)). Both directories will be removed (cleaned up) after test execution - independent of whether the test execution succeeds or fails.
  - *Exception*: If the `--work-directory` command-line parameter is provided, then this directory will not be cleaned up (deleted) after execution. A temporary directory, that will be removed after execution, will be created within the working directory.
- **Process Isolation**: Scrut starts individual `bash` processes for executing each testcase of the same test file. Each shell expression. The environment of the previous execution is pulled in through a shared `state` file, that contains all environment variables, shell variables, aliases, functions and settings as they were set when the the previous testcase execution ended.
  - *Exception*: All testcases in cram files are currently executed within the same `bash` process - this is likely to change in the future.
