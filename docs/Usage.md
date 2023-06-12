# Usage

Scrut is intended to be used by authors of CLIs to create, maintain and run integration tests. Furthermore, Scrut is perfectly suitable to automatically run previously created tests embedded in CI/CD pipelines.

Check out the help to see an overview of all available commands of the `scrut` binary:

```sh
$ scrut help
A testing toolkit to scrutinize CLI applications

Usage: scrut [OPTIONS] <COMMAND>

Commands:
  create
          Create tests from provided shell expression
  test
          Run tests from files or directories
  update
          Re-run all testcases in given file(s) and update the output
          expectations
  help
          Print this message or the help of the given subcommand(s)

Options:
  -C, --cram-compat
          Do things to be as compatible as possible with Cram: Inject CRAM*
          environment variables. Use glob matcher that supports escaped
          wildcards. Enable the --combine-output parameter. Enable the
          --keep-output-crlf parameter

      --combine-output
          Per default only STDOUT will be considered. This flags combines STDOUT
          and STDERR into a single stream

      --keep-output-crlf
          Per default all CRLF line endings from outputs of shell expressions
          will be converted into LF line endings and need not be considered in
          output expectations. This flag surfaces CRLF line endings so that they
          can (and must be) addressed in output expectations (e.g. `output
          line\r (escaped)`)

  -e, --escaping <ESCAPING>
          Optional output escaping mode. If not set then defaults to escaping
          all non-printable unicode characters for Scrut Markdown tests and all
          non-printable ASCII characters for Cram tests

          Possible values:
          - ascii:
            All non ASCII and all non-printable ASCII characters are escaped
          - unicode:
            All non-printable Unicode characters are escaped

  -s, --shell <SHELL>
          Shell to execute expressions in

          [default: /bin/bash]

  -w, --work-directory <WORK_DIRECTORY>
          Optional path to work directory in which the tests will be executed.
          Per default a temporary work directory for each test file will be
          created instead

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## `scrut test <FILES>`

The quintessential command of `scrut`, which runs tests in one or multiple files or directories.

### TL;DR

Run all tests in the [`selftest/commands/create.md`](selftest/commands/create.md) file:

```sh
$ scrut test selftest/commands/create.md
Summary: 1 file(s) with 2 test(s): 2 succeeded, 0 failed and 0 skipped
```

Run all tests in all files in the [`selftest/cases`](selftest/cases) folder:

```sh
$ scrut test selftest/cases
Summary: 11 file(s) with 66 test(s): 63 succeeded, 0 failed and 3 skipped
```

Run all all `sh` code blocks in the documentation you are currently reading:

```bash
$ scrut test --markdown-languages sh --work-directory "$(pwd)" README.md docs
```

> **Note**: To understand why the `--markdown-languages` was specified - and why the command is not ending in infinity recursion - look up the source code of this very file ..

### Help

```sh
$ scrut test -h
Run tests from files or directories

Usage: scrut test [OPTIONS] [TEST_FILE_PATHS]...

Arguments:
  [TEST_FILE_PATHS]...  Path to test files or directories

Options:
  -P, --prepend-test-file-paths [<PREPEND_TEST_FILE_PATHS>...]
          Optional list of paths to test files which are prepended to each test
          file in execution. Think: shared test bootstrap. This is NOT meant to
          be used from the command line, aside from edge-cases, for it breaks
          the containment of test files. Use the configuration file and persist
          it together with your tests instead. UNSTABLE: this parameter may
          change or be removed
  -A, --append-test-file-paths [<APPEND_TEST_FILE_PATHS>...]
          Optional list of paths to test files which are appended to each test
          file in execution. Think: shared test teardown. This is NOT meant to
          be used from the command line, aside from edge-cases, for it breaks
          the containment of test files. Use the configuration file and persist
          it together with your tests instead. UNSTABLE: this parameter may
          change or be removed
      --debug
          Whether to print out debug output - use only
  -C, --cram-compat
          Do things to be as compatible as possible with Cram: Inject CRAM*
          environment variables. Use glob matcher that supports escaped
          wildcards. Enable the --combine-output parameter. Enable the
          --keep-output-crlf parameter
  -L, --markdown-languages <MARKDOWN_LANGUAGES>...
          For markdown format: Language annotations that are considered test
          cases [default: scrut testcase]
      --combine-output
          Per default only STDOUT will be considered. This flags combines STDOUT
          and STDERR into a single stream
      --match-cram <MATCH_CRAM>
          Glob match that identifies cram files [default: *.{t,cram}]
      --keep-output-crlf
          Per default all CRLF line endings from outputs of shell expressions
          will be converted into LF line endings and need not be considered in
          output expectations. This flag surfaces CRLF line endings so that they
          can (and must be) addressed in output expectations (e.g. `output
          line\r (escaped)`)
      --match-markdown <MATCH_MARKDOWN>
          Glob match that identifies markdown files [default: *.{md,markdown}]
  -e, --escaping <ESCAPING>
          Optional output escaping mode. If not set then defaults to escaping
          all non-printable unicode characters for Scrut Markdown tests and all
          non-printable ASCII characters for Cram tests [possible values: ascii,
          unicode]
      --no-color
          Per default colo(u)r output is enabled on TTYs when the `diff`
          renderer is used. This flag disables colo(u)r output in that case
  -r, --renderer <RENDERER>
          Which renderer to use for generating the result, with `diff` being the
          best choice for human consumption and `json` or `yaml` for further
          machine processing [default: auto] [possible values: auto, pretty,
          diff, json, yaml]
  -s, --shell <SHELL>
          Shell to execute expressions in [default: /bin/bash]
      --absolute-line-numbers
          Per default, renderers that provide line numbers use relative numbers
          within the test case / the output of the execution. Setting this flag
          changes that to use absolute line numbers from within the test file
  -w, --work-directory <WORK_DIRECTORY>
          Optional path to work directory in which the tests will be executed.
          Per default a temporary work directory for each test file will be
          created instead
  -S, --timeout-seconds <TIMEOUT_SECONDS>
          For sequential: Timeout in seconds for whole execution. Use 0 for
          unlimited [default: 900]
  -h, --help
          Print help (see more with '--help')
```

## `scrut create <COMMAND>`

Who likes toil? I don't. The `create` command makes it less bothersome to create new tests.

### TL;DR

Create a simple Markdown encoded test:

````bash
$ scrut create "echo Hello World"
# Command executes successfully

```scrut
$ echo Hello World
Hello World
```
````

Create a test that tests `scrut help` itself

```sh
$ echo "scrut help" | scrut create -o /tmp/test-scrut-help.md -
```

> **Note**: This could now tested with `scrut test /tmp/test-scrut-help.md`

### Help

```sh
$ scrut create -h
Create tests from provided shell expression

Usage: scrut create [OPTIONS] <SHELL_EXPRESSION>...

Arguments:
  <SHELL_EXPRESSION>...  Shell expression THAT WILL BE EXECUTED to automatically
                         create a test from. Use "-" to read from STDIN

Options:
  -f, --format <FORMAT>
          What kind of test format to create [default: markdown] [possible
          values: markdown, cram]
  -o, --output <OUTPUT>
          Where to output the created test to (STDOUT is "-") [default: -]
  -t, --title <TITLE>
          What the test is supposed to prove [default: "Command executes
          successfully"]
  -C, --cram-compat
          Do things to be as compatible as possible with Cram: Inject CRAM*
          environment variables. Use glob matcher that supports escaped
          wildcards. Enable the --combine-output parameter. Enable the
          --keep-output-crlf parameter
  -S, --timeout-seconds <TIMEOUT_SECONDS>
          Max execution time for the provided shell expression to execute
          [default: 900]
      --combine-output
          Per default only STDOUT will be considered. This flags combines STDOUT
          and STDERR into a single stream
      --keep-output-crlf
          Per default all CRLF line endings from outputs of shell expressions
          will be converted into LF line endings and need not be considered in
          output expectations. This flag surfaces CRLF line endings so that they
          can (and must be) addressed in output expectations (e.g. `output
          line\r (escaped)`)
  -e, --escaping <ESCAPING>
          Optional output escaping mode. If not set then defaults to escaping
          all non-printable unicode characters for Scrut Markdown tests and all
          non-printable ASCII characters for Cram tests [possible values: ascii,
          unicode]
  -s, --shell <SHELL>
          Shell to execute expressions in [default: /bin/bash]
  -w, --work-directory <WORK_DIRECTORY>
          Optional path to work directory in which the tests will be executed.
          Per default a temporary work directory for each test file will be
          created instead
  -h, --help
          Print help (see more with '--help')
```

## `scrut update <FILE>`

Having tests coverage for your CLI is amazing. Continuously developing or occasionally updating your CLI may result in toil work updating your tests. To keep that toil as low as possible the `update` command is your friend.

### TL;DR

Given an outdated (aka invalid) test:

```sh
$ echo -e '# A test\n\n```scrut\n$ echo Hello\nWorld\n```\n' > /tmp/test.md
```

Update is just one command:

```sh
$ scrut update --assume-yes /tmp/test.md 2>&1
// =============================================================================
// @ /tmp/test.md:4
// -----------------------------------------------------------------------------
// # A test
// -----------------------------------------------------------------------------
// $ echo Hello
// =============================================================================

   1  | - World
1     | + Hello


Summary: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
Summary: 1 file(s) of which 1 updated, 0 skipped and 0 unchanged
```

This creates a new file `/tmp/test.md.new`, which has the updated test:

```sh
$ diff /tmp/test.md /tmp/test.md.new
5c5
< World
---
> Hello
[1]
```

> **Note**: You could use `--replace` to directly overwrite the `test.md` file.

### Update as mass create

While `scrut create` is helpful to create a new test for a single command, often your tests may be more complex, consisting of multiple executions. If that is the case, just create a new Markdown file and fill it with all the commands, but omit all outputs. You then can use `scrupt update <file.md> --replace` to "fill in the blanks", so to speak.

For example, consider the following Markdown file that has all the tests, but is lacking the test output:

    # A test suite of multiple tests

    ## Do something

    ```scrut
    $ export FOO_TEXT='Hello World'
    ```

    ## Echo the hostname

    ```scrut
    $ echo "$FOO_TEXT"
    ```

    ## Yet more tests ...

    ```scrut
    $ echo -ne "Some\nTest\nOutput"
    ```

Assuming the above is stored in `tests.md`, updating it with `scrut update tests.md --replace` will fill in the outputs and result in:

    # A test suite of multiple tests

    ## Do something

    ```scrut
    $ export FOO_TEXT='Hello World'
    ```

    ## Echo the hostname

    ```scrut
    $ echo "$FOO_TEXT"
    Hello World
    ```

    ## Yet more tests ...

    ```scrut
    $ echo -ne "Some\nTest\nOutput"
    Some
    Test
    Output (no-eol)
    ```

This is the quickest way to create complex test cases without having to copy & paste lots of outputs from the command line to a text editor.

### Help

```sh
$ scrut update -h
Re-run all testcases in given file(s) and update the output expectations

Usage: scrut update [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Path to test files or directories

Options:
      --debug
          Whether to print out debug output - use only
  -L, --markdown-languages <MARKDOWN_LANGUAGES>...
          For markdown format: Language annotations that are considered test
          cases [default: scrut testcase]
      --no-color
          Per default colo(u)r output is enabled on TTYs when the `diff`
          renderer is used. This flag disables colo(u)r output in that case
  -C, --cram-compat
          Do things to be as compatible as possible with Cram: Inject CRAM*
          environment variables. Use glob matcher that supports escaped
          wildcards. Enable the --combine-output parameter. Enable the
          --keep-output-crlf parameter
  -o, --output-suffix <OUTPUT_SUFFIX>
          What suffix to add to thew newly created file (will overwrite already
          existing files!) [default: .new]
      --combine-output
          Per default only STDOUT will be considered. This flags combines STDOUT
          and STDERR into a single stream
  -y, --assume-yes
          Danger! Whether to assume Yes for the question to overwrite files when
          with updated contents. In conjunction with the `--replace` flag this
          means the original file will be overwritten
      --keep-output-crlf
          Per default all CRLF line endings from outputs of shell expressions
          will be converted into LF line endings and need not be considered in
          output expectations. This flag surfaces CRLF line endings so that they
          can (and must be) addressed in output expectations (e.g. `output
          line\r (escaped)`)
      --match-cram <MATCH_CRAM>
          Glob match that identifies cram files [default: *.{t,cram}]
  -e, --escaping <ESCAPING>
          Optional output escaping mode. If not set then defaults to escaping
          all non-printable unicode characters for Scrut Markdown tests and all
          non-printable ASCII characters for Cram tests [possible values: ascii,
          unicode]
      --match-markdown <MATCH_MARKDOWN>
          Glob match that identifies markdown files [default: *.{md,markdown}]
  -r, --replace
          Whether to replace the contents of the files (see --output-suffix)
  -s, --shell <SHELL>
          Shell to execute expressions in [default: /bin/bash]
  -S, --timeout-seconds <TIMEOUT_SECONDS>
          For sequential: Timeout in seconds for whole execution. Use 0 for
          unlimited [default: 900]
  -w, --work-directory <WORK_DIRECTORY>
          Optional path to work directory in which the tests will be executed.
          Per default a temporary work directory for each test file will be
          created instead
      --absolute-line-numbers
          Per default, renderers that provide line numbers use relative numbers
          within the test case / the output of the execution. Setting this flag
          changes that to use absolute line numbers from within the test file
  -c, --convert <CONVERT>
          Optional explicit format, in case the intention is to convert a test.
          If set then --output-suffix is ignored (new format file extension is
          used instead). Has no effect if the same format is chosen that the
          input test file already has [possible values: markdown, cram]
  -h, --help
          Print help (see more with '--help')
```
