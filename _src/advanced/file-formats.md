---
sidebar_position: 1
---


# File Formats

Scrut supports multiple test file formats. The recommended format is [Markdown](#markdown-format).

## File Anatomy

All test files contain one or more test cases. There are two common patterns to structure test files in Scrut:

- **Coherent Test Suite** (recommended): One test file represents one use-case or behavior. This makes it easy to identify broken functionality.
- **List of Tests**: One test file contains a list of simple, not necessarily related tests.

Markdown files support [document wide configuration](#inline-configuration) in the form of "YAML Frontmatter".

### Test Case Anatomy

Each individual test that lives in a test file is called a _Test Case_ and consists of the following components:

1. A **Title**, so that a human can understand what is being done
2. A **Shell Expression**, that can be anything from a single command to a multi-line, multi-piped expression
3. **[Expectations](expectations.md)** of the output that the Shell Expression will yield
4. Optionally the expected _Exit Code_ the Shell Expression must end in - if anything but successful execution (`0`) is expected
5. Optionally per-test-case configuration (only supported by Markdown format)

## Markdown Format

[Markdown](https://www.markdownguide.org/) is an amazingly simple, yet powerful language. To write _Test Cases_ in Markdown follow this guidance:

- _Shell Expressions_ and _Expectations_ live in the same code-block, that must be annotated with the language `scrut`
  - The first line of a _Shell Expressions_ must start with `$ ` (dollar, sign followed by a space), any subsequent with `> ` (closing angle bracket / chevron, followed by a space)
  - All other lines in the code block (including empty ones) that follow the _Shell Expression_ are considered _Expectations_
  - Lines starting with `#` that precede the shell expression are ignored (comments)
  - If an _Exit Code_ other than 0 is expected, it can be denoted in square brackets `[123]` once per _Test Case_
- The first line before the code block that is either a paragraph or a header will be used as the _Title_ of the _Test Case_

Here an example:

````markdown
This is the title

```scrut
$ command | \
>   other-command
expected output line
another expected output line
[123]
```
````

The following **constraints** apply:

- A markdown file can contain as many Test Cases as needed (1..n)
- Each code block in a Test Case may only have _one_ (1) Shell Expression (each Test Case is considered atomic)
- Code blocks that do not denote a language (or a different language than `scrut`) will be ignored

With that in mind, consider the following markdown file that contains not only Test Cases but arbitrary other text and other code blocks. This is idiomatic Scrut markdown files that combines tests and documentation:

````
# This is just regular markdown

It contains both Scrut tests **and**  abitrary text, including code examples,
that are unrelated to Scrut.

```python
import os

print("This code block ignored by Scrut")
```

## Here is a scrut test

```scrut
$ echo Hello
Hello
```

## Embedded with other documentation

So it's a mix of test and not tests.

Any amount of tests are fine:

```scrut
$ echo World
World
```

Just make sure to write only one Test Case per code-block.
````

> **Note**: If you are testing actual markdown output, be aware that you can embed code blocks in other code blocks, if the outer code block uses one more backtick (opening and closing!) than the embedded one(s). Just have a look at the source code of this file right above this text.

### Inline Configuration

Scrut supports two kinds of inline configuration:

1. **Per Document** (document-wide) configuration, which can be defined at the start of the test file
2. **Per Test Case** (test-case-wide) configuration, which can be defined with each individual Test Case

**Example**

````markdown
---
# document-wide YAML configuration
total_timeout: 30s
---

# The test document

The initial block that is initialized with `---` and terminated with `---` contains the configuration in YAML notation.

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

Some inline-configuration attribute can overwritten by parameters provided on the command-line. The order of precedence is:
1. Command-line parameter
2. Per-TestCase configuration
3. Per-Document configuration
4. Default

#### Document Configuration

| Name            | Type                                                           | Corresponding Command Line Parameter | Description                                                                                                                                                                                                                                      |
| --------------- | -------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `append`        | list of strings                                                | `--append-test-file-paths`           | Include these paths in order, as if they were part of this file. All tests within the appended paths are appended to the tests defined in this file. Use-case is common/shared test tear-down. Paths must be relative to the current `$TESTDIR`. |
| `defaults`      | [TestCase Configuration](#testcase-configuration)              | n/a                                  | Defaults for per-test-case configuration within the test file.                                                                                                                                                                                   |
| `prepend`       | list of strings                                                | `--prepend-test-file-paths`          | Include these paths in order, as if they were part of this file. All tests within the prepend paths are prepended to the tests defined in this file. Use-case is common/shared test setup. Paths must be relative to the current `$TESTDIR`.     |
| `shell`         | string                                                         | `--shell`                            | The path to the shell. If a full path is not provided, then the command must be in `$PATH`. **Only `bash` compatible shells are currently supported!**                                                                                           |
| `total_timeout` | [duration string](https://docs.rs/humantime/latest/humantime/) | `--timeout-seconds`                  | All tests within the file (including appended and prepended) must finish executing within this time.                                                                                                                                             |

**Defaults (Markdown and Cram)**

```yaml
append: []
defaults: {}
prepend: []
shell: bash
total_timeout: 15m
```

**Caveats**

- Per-document configuration in files that are appended or prepended is ignored

#### TestCase Configuration

| Name                 | Type                                                                                                                | Corresponding Command Line Parameter         | Description                                                                                                                                                                                                                                                                                                                              |
| -------------------- | ------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `detached`           | boolean                                                                                                             | n/a                                          | Tell Scrut that the shell expression of this test will detach itself, so Scrut will not consider this a test (i.e. no output or exit code evaluation). Purpose is to allow the user to detach a command (like `nohup some-command &`) that is doing something asynchronous (e.g. starting a server to which the tested CLI is a client). |
| `environment`        | object                                                                                                              | n/a                                          | A set of environment variable names and values that will be explicitly set for the test.                                                                                                                                                                                                                                                 |
| `keep_crlf`          | boolean                                                                                                             | `--keep-output-crlf`                         | Whether CRLF should be translated to LF (=false) or whether CR needs to be explicitly handled (=true).                                                                                                                                                                                                                                   |
| `output_stream`      | enum (`stdout`, `stderr`, `combined`)                                                                               | `--combine-output` and `--no-combine-output` | Which output stream to choose when applying output expectations: `stdout` (all expectations apply to what is printed on STDOUT), `stderr` (all expectations apply to what is printed on STDERR), `combined` (STDOUT and STDERR will combined into a single stream where all expectations are applied on)                                 |
| `skip_document_code` | positive integer                                                                                                    | n/a                                          | The exit code, that if returned by any test, leads to skipping of the whole file.                                                                                                                                                                                                                                                        |
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

## Cram Format

Also supported, for compatibility, is the Cram file format. The general guidance to write _Test Cases_ in Cram files is:

- The first line of _Shell Expression_ must start with `  $ ` (space + space + dollar + space), any subsequent with `  > ` (space + space + closing angle bracket + space)
  - This is slightly different from classic scrut syntax. Be mindful of the additional spaces
- Lines following the _Shell Expression_, that are also indented with two spaces, are considered _Expectations_
  - If an Exit Code other than 0 is expected, it can be denoted in square brackets ` [123]` once per Test Case
  - Note: Empty output lines (=empty _Expectations_) must still have two leading space characters
  - Note: A fully empty line (no leading spaces) denotes the end of the current _Test Case_
- If the _Shell Expression_ is preceded by a non-empty line (that is _not_ indented) the line is considered the _Title_ of the _Test Case_

Here an example:

```cram
This is a comment
  $ scrut --help
  Scrut help output

Another Test Case in the same file
  $ scrut --version
  Scrut version output
```

Multiple tests Test Cases can be written in sequence, without any empty lines in between:

```cram
A title for the first Test Case
  $ first --command
  $ second --command
  $ third --comand
  Output Expectation
```

> **Note**: Remember the indenting space characters!

## Which format to chose?

We recommend the Markdown format which was introduced with two goals in mind:

1. **Tests ❤️ Documentation**: The value of tests is not only in proving behavior, but also in documenting it - and thereby also in teaching it. The Markdown Test Case format allows you to keep tests around in a way that future generations of maintainers will love you for.
2. **Bad Spaces 👾**: To denote an expected empty line of output in Cram format you have to provide two empty spaces ` `. This goes counter a lot of default behavior in the development toolchain. Many CI/CD tools are tuned to automatically ignore changes that only pertain spaces. Code review tools often deliberately hide those changes. Spaces are generally hard to see in code editors - if they are visualized at all. Breaking tests that are caused by an accidentally removed or added space cause rage quitting.

If these arguments resonate with you, go for the Markdown format. If not you are probably better of with Cram that allows for a more condensed writing style. Choices, choices.
