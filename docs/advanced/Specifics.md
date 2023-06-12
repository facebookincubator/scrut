# Specifics

This chapter describes behaviors of Scrut that should be known by the user to prevent surprises in the wrong moment.

## Test output

Executing a test with Scrut results either in success (when all expectations in the test match) or failure (when at least one expectation in the test does not match).

In case of success, Scrut does not print out anything:

```bash
$ scrut test /path/to/a-working-test.md
```

In case of failure, Scrut will point you towards the problems. Multiple output modes, which are called renderers, are supported. Currently the `diff` renderer is the default. The output of a failed expectation looks like this:

```
$ scrut test /path/to/a-working-test.md
// =============================================================================
// @ /path/to/a-failing-test.md:4
// -----------------------------------------------------------------------------
// # A test
// -----------------------------------------------------------------------------
// $ echo Hello
// =============================================================================

1  1  |   Foo
   2  | - Bar
2  3  |   Baz
3     | + Zoing
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
- `   2  | - Bar`: This line was expected, but not printed. The left hand omitted number indicates that it was not found in output. The right hand number tells that this is the second expectation. The `-` before the line `Bar` emphasizes that this is a missed expectation.
- `2  3  |   Baz`: This line was printed and expected. The left hand `2` is the number of the output line and the right hand `3` is the number of the expectation.
- `3     | + Zoing`: This line was printed unexpectedly. The left hand `3` is the number of the output line the omitted right hand number implies there is no expectation that covers it. The `+` before the line `Zoing` emphasizes that this is a "surplus" line.

> **Note**: If you work with test files that contain a large amount of tests, then you may want to use the `--absolute-line-numbers` flag on the command line: instead of printing the relative line number for each test, as described above, it prints absolute line numbers from within the test file. Assuming the `Foo` expectation from above is in line 10 of a file, it would read `10  10  |   Foo` - and all subsequent output liens with respective aligned line numbers.

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

The directory within which tests are being executed can be explicitly set using the `--work-directory` parameter for the `test` and `update` commands. If that parameter is set then _all tests_ from _all test files_ are executed run within that directory, and the directory is _not removed_ afterwards.

## Execution

`scrut test` executes all provided test files in sequential order. All tests within each file are executed from the top down in sequential order.

### Shared State

All tests within the same file are executed in the _same_ shell process. That means they share a state. Consider the following:

````markdown
# Setup some env variable

```scrut
$ export FOO=bar
```

# use the env variable

```scrut
$ echo "Bar is $FOO"
Bar is bar
```
````

The exported variable from the first test will be available to the second test. So the output will be `Bar is bar`.

> **Note**: If you want to share aliases using `/bin/bash` mind that you need to `shopt -s expand_aliases` (likely in a `setup.sh` bootstrap file) beforehand, as in:
>
> ````markdown
> Enable alias sharing (expanding)
>
> ```scrut
> $ shopt -s expand_aliases
> ```
>
> Expand an alias
>
> ```scrut
> $ alias foo="echo FOO"
> ```
>
> Use expanded alias
>
> ```scrut
> $ foo
> FOO
> ```
> ````

## Test isolation

`scrut` itself only isolates executions [in a temporary directory](#test-work-directory). If you need to isolate test execution further, consider running your tests in a jail, container or VM. You can do that by instrumenting the `--shell` parameter.

In order to understand how, have a peek under the hood of `scrut` to see how shell expressions of tests are executed. Consider the following:

```bash
$ echo "echo Hello" | /bin/bash -
Hello
```

What the above does is piping the string `echo Hello` into the `STDIN` of the process that was started with `/bin/bash -`. This is exactly what `scrut test` does.

Knowing this, you could easily create a wrapper for `bash`:

```bash
#!/bin/bash

# do something in this wrapper script
printenv > /tmp/env

# execute bash again, so that it expects STDIN
/bin/bash -
```

Assuming the above is stored in `/usr/local/bash-wrapper` and executable, you could just provide it to `scrut test --shell /usr/local/bash-wrapper ...`. Instead of just printing the env vars into a file and staring bash again, you could, for example, start `bash` in a docker container or `ssh` into a machine.

> **Note**: If your the shell expressions of your test(s) reference other files (as in `source "$TESTDIR"/setup.sh` or so): make sure to have those files available in the same location in your execution environment as they would be locally (or use a custom environment variable)

> **Caution**: Scrut is primarily developed and tested with `/bin/bash` in mind. Other shells may come with breaking behavior. Scrut especially cannot deal with shells that echo the commands out that are piped into it (e.g. [`script`](https://linux.die.net/man/1/script)). Active development - things may improve.

# STDOUT and STDERR

[Expectations](Advanced/Expectations.md) always only test [the primary output of a CLI program](https://clig.dev/#:~:text=primary%20output%20for%20your%20command). This means: only `STDOUT` is considered.

To make that clear: Assuming you have a command `foo` that outputs `Hello` on `STDOUT` and `World` on `STDERR` then the following test will suffice:

````
```scrut
# Only STDOUT is considered
$ foo
Hello
```
````

If you need to test the output of `STDERR` instead of `STDOUT`, you can drop `STDOUT` and redirect `STDERR` to it like so:

````
```scrut
# Only STDERR is considered
$ foo 2>&1 1>/dev/null
World
```
````

Of course, you can combine the output of either:

````
```scrut
# Only STDERR is considered
$ foo 2>&1
Hello
World
```
````

> **Caution**: The order of the output is not guaranteed.

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

Unless otherwise specified an exit code of 0 (zero) is assumed. You can explicitly denote it, but why?

> **Note**: Exit code evaluation happens before output expectations are evaluated

### Skip Tests with Exit Code 80

If any testcase in a test file exist with exit code `80`, then all testcases in that file are skipped.

This is especially helpful for OS specific tests etc. Imagine:

````
Run tests in this file only on Mac

```scrut
$ [[ "$(uname)" == "Darwin" ]] || exit 80
```
````

### Scrut Exit Code

Scrut itself communicates the outcome of executions with exit codes. Currently three possible exit codes are supported:

- `0`: Command succeeded, all is good (`scrut test`, `scrut create`, `scrut update`)
- `1`: Command failed with error (`scrut test`, `scrut create`, `scrut update`)
- `50`: Validation failed (`scrut test` only)

## Newline handling

[Newline](https://en.wikipedia.org/wiki/Newline) endings is a sad story in computer history. In Unix / MacOS ( / \*BSD / Amiga / ..) the standard line ending is the line feed (LF) character `\n`. Windows (also Palm OS and OS/2?) infamously attempted to make a combination of carriage return (CR) and line feed the standard: CRLF (`\r\n`). Everybody got mad and still is.

Scrut internally _only_ works with LF. At the input boundaries of reading test files and reading output from test command execution Scrut transforms any CRLF into LF. Currently Scrut does _not output_ any CRLF anywhere.
