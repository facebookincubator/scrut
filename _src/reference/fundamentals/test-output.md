# Test Output

Executing a test document with Scrut results either in success (when all expectations in the test match) or failure (when at least one expectation in the test document does not match).

Scrut supports multiple *output renderers*, which yield a different representation of these test results.

## Pretty Renderer (default)

Scrut will always tell you what it did:

```bash title="Terminal"
$ scrut test selftest/cases/regex.md
🔎 Found 1 test document(s)

Result: 1 document(s) with 10 testcase(s): 10 succeeded, 0 failed and 0 skipped
```

In case of failure the `pretty` default renderer will provide a human-readable output that points you to the problem with the output:

```bash title="Terminal"
$ scrut test a-failing-test.md
🔎 Found 1 test document(s)
❌ /tmp/test.md: failed 1 out of 1 testcase

// =============================================================================
// @ a-failing-test.md:10
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


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
```

The failure output consists of two components:

1. The failure header, which consists of all initial lines that start with `//`, indicates the position
2. The failure body, which consists of all the following lines, indicates the problem

**Header**

The header contains three relevant information. Given the above output:

- `@ a-failing-test.md:10`, tells you that the test that failed is in the provided document `a-failing-test.md` and that the [shell expression](/docs/reference/fundamentals/shell-expression/) (that failed the test) starts in line ten of that file.
- `# <test title>`, gives you the optional title of the test in the document. *If the test does not have a title, this line is omitted.*
- `$ <test command>`, is the [shell expression](/docs/reference/fundamentals/shell-expression/) from the [test document](/docs/reference/fundamentals/test-document/) that is tested and that has failed.

:::note

See [Reference > Fundamentals > Test Case](/docs/reference/fundamentals/test-case/)) to learn more about test case anatomy.

:::

**Body**

There are two possible variants that the `diff` renderer may return:

1. Failed [output expectations](/docs/reference/fundamentals/output-expectations/)
2. Failed [exit code expectation](/docs/reference/behavior/exit-codes/)

The above output is a failed [output expectations](/docs/reference/fundamentals/output-expectations/) and you can read it as following:

- `1  1  |   Foo`: This line was printed as expected. The left hand `1` is the number of the output line and the right hand `1` is the number of the expectation.
- `   2  | - BAR`: This line was expected, but not printed. The left hand omitted number indicates that it was not found in output. The right hand number tells that this is the second expectation. The `-` before the line `Bar` emphasizes that this is a missed expectation.
- `2     | + Bar`: This line was printed and expected. The left hand `2` is the number of the output line and the right hand `3` is the number of the expectation.
- `3     | + Baz`: This line was printed unexpectedly. The left hand `3` is the number of the output line the omitted right hand number implies there is no expectation that covers it. The `+` before the line `Zoing` emphasizes that this is a "surplus" line.

:::note

If you work with test files that contain a large amount of tests, then you may want to use the `--absolute-line-numbers` flag on the command line: instead of printing the relative line number for each test, as described above, it prints absolute line numbers from within the test file. Assuming the `Foo` expectation from above is in line 10 of a file, it would read `13  13  |   Foo` - and all subsequent output liens with respective aligned line numbers.

:::

An example for the body of an *exit code expectation*:

```
unexpected exit code
  expected: 2
  actual:   0

## STDOUT
#> Foo
## STDERR
```

This should be mostly self-explanatory. Scrut does not provide any [output expectation](/docs/reference/fundamentals/output-expectations/) failures, because it assumes that when the exit code is different, then it is highly likely that the output is very different - and even if not, it would not matter, as it failed anyway.

The tailing `## STDOUT` and `## STDERR` contain the output lines (prefixed with `#> `) that were printed out from the failed execution.

### Multiline Matches

If you use output expectations with a [quantifier](/docs/reference/fundamentals/output-expectations/#quantifiers) that allows for multiline matches then Scrut will print the output lines that match the expectation. For example, consider the following test:

````markdown title="some-test.md" showLineNumbers
# Some test

```scrut
$ echo -e "foo\nbar1\nbar2\nbar3\nbar4\nbaz"
bar* (glob+)
```
````

This test will fail, because `bar* (glob+)` does not mach the first or the last line. The failed output will look like this:

```bash title="Terminal"
$ scrut test some-test.md
🔎 Found 1 test document(s)
❌ some-test.md: failed 1 out of 1 testcase

// =============================================================================
// @ some-test.md:4
// -----------------------------------------------------------------------------
// # Some test
// -----------------------------------------------------------------------------
// $ echo -e "foo\nbar1\nbar2\nbar3\nbar4\nbaz"
// =============================================================================

   1  | + foo
1+ 2  |   bar1  // bar* (glob+)
 + 3  |   bar2
 + 4  |   bar3
 + 5  |   bar4
   6  | + baz


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
```

:::note

The amount of lines that are printed is controlled by the `--multiline-match-lines` flag. The default is `100` to strike a balance between meaningful output and flooding the terminal. If more lines than that are produced then only the first 50 and last 50 lines are printed, with a hint that there are more lines in between:

```bash title="Terminal"
$ scrut test --max-multiline-matched-lines=2 some-test.md
🔎 Found 1 test document(s)
❌ some-test.md: failed 1 out of 1 testcase

// =============================================================================
// @ some-test.md:4
// -----------------------------------------------------------------------------
// # Some test
// -----------------------------------------------------------------------------
// $ echo -e "foo\nbar1\nbar2\nbar3\nbar4\nbaz"
// =============================================================================

   1  | + foo
1+ 2  |   bar1  // bar* (glob+)
 +    | …
 + 5  |   bar4
   6  | + baz


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
```

If you set `--multiline-match-lines` to less or equal `1` then Scrut will not print any lines that match the expectation, but only the expectation itself:

```
   1  | + foo
1+ +  |   bar* (glob+)
   6  | + baz
```

:::

## Diff renderer

The `diff` renderer, that can be enabled with `--renderer diff` (or `-r diff`), prints a diff in the [unified format](https://en.wikipedia.org/wiki/Diff#Unified_format).

```bash title="Terminal"
$ scrut test -r diff a-failing-test.md
🔎 Found 1 test document(s)
❌ a-failing-test.md: failed 1 out of 1 testcase

--- a-failing-test.md
+++ a-failing-test.md.new
@@ -14 +14,2 @@ malformed output: One conjunct expression
-BAR
+Bar
+Baz
```

:::tip

The created diff is compatible with the `patch` command line tool (e.g. `patch -p0 < <(scrut test -r diff a-failing-test.md)`). This is mostly equivalent to using the `scrut update` command.

:::

## JUnit renderer

The `junit` renderer, that can be enabled with `--renderer junit` (or `-r junit`), prints a JUnit XML report, the format that CI test result collectors (GitHub Actions, Jenkins, GitLab, ...) consume.

The report is written to STDOUT, and progress and log messages to STDERR, so it can be redirected into a file:

```bash title="Terminal"
$ scrut test -r junit tests/example.md > report.xml
```

Given a document with one passing and one failing test case, the report reads:

```xml title="report.xml"
<?xml version="1.0" encoding="UTF-8"?>
<testsuites tests="2" failures="1" errors="0" skipped="0" time="0.030">
  <testsuite name="tests/example.md" package="tests" timestamp="2026-09-29T09:41:57" tests="2" failures="1" errors="0" skipped="0" time="0.030">
    <properties>
      <property name="scrut.format" value="markdown"/>
    </properties>
    <testcase name="Greets the world" classname="tests/example.md" file="tests/example.md" line="6" time="0.014">
      <system-out>Hello World</system-out>
    </testcase>
    <testcase name="Prints the wrong thing" classname="tests/example.md" file="tests/example.md" line="14" time="0.015">
      <failure type="malformed_output" message="output does not match expectations">// =============================================================================
// @ tests/example.md:13
// -----------------------------------------------------------------------------
// # Prints the wrong thing
// -----------------------------------------------------------------------------
// $ echo &quot;Hello Moon&quot;
// =============================================================================

1     | - Hello World
   1  | + Hello Moon</failure>
      <system-out>Hello Moon</system-out>
    </testcase>
  </testsuite>
</testsuites>
```

**Test suites**

Each [test document](/docs/reference/fundamentals/test-document/) becomes one `<testsuite>`:

- `name` is the path of the document, relative to the current working directory and with `/` separators on all platforms. A document outside the working directory keeps the path it was given.
- `package` is the directory of the document. It is omitted for documents in the working directory itself.
- `timestamp` is the local time, without offset, at which the run finished. All suites of one run carry the same value.
- The `scrut.format` property is the format of the document: `markdown` or `cram`.

**Test cases**

Each [test case](/docs/reference/fundamentals/test-case/) becomes one `<testcase>`:

- `name` is the title of the test case, or `line <number>` if it has none. Duplicate titles within a document get a numeric suffix (`My test #2`), because report consumers identify test cases by `classname` and `name`.
- `classname` and `file` are the path of the document, as in the suite `name`.
- `line` is the line of the first [output expectation](/docs/reference/fundamentals/output-expectations/) that did not match, so that annotations point at the failing line. For all other outcomes it is the line of the [shell expression](/docs/reference/fundamentals/shell-expression/).
- `time` is the execution duration in seconds. It is omitted when the duration is unknown, which is the case for Cram documents and with `--cram-compat`.
- `<system-out>` and `<system-err>` contain the captured output. Non-printable characters are escaped.

**Outcomes**

| Outcome | Element | `type` |
| --- | --- | --- |
| Success | (none) | |
| Output did not match | `<failure>` | `malformed_output` |
| Unexpected [exit code](/docs/reference/behavior/exit-codes/) | `<failure>` | `invalid_exit_code` |
| [JSON Schema](/docs/reference/fundamentals/validation-modes/) invalid, output not JSON, or output does not conform | `<failure>` | `json_schema_invalid_schema`, `json_schema_invalid_json`, `json_schema_validation_errors` |
| [Interactive](/docs/reference/fundamentals/interactive-mode/) directive failed | `<failure>` | `interactive_failed` |
| Timeout | `<error>` | `timeout` |
| Internal error | `<error>` | `internal_error` |
| Not executed | `<skipped>` | |

The body of `<failure>` and `<error>` is the same explanation the `pretty` renderer prints. A test case is not executed if a test case in the document exits with the [skip exit code](/docs/reference/behavior/exit-codes/#skip-tests-with-exit-code-80), or if an earlier test case timed out or failed with [`fail_fast`](/docs/reference/fundamentals/inline-configuration/#fail_fast) enabled.

:::tip

`scrut test` still exits with code `50` when a test fails. In CI, make sure the step that uploads or publishes `report.xml` also runs when the test step failed (e.g. `if: always()` or `if: failure()` in GitHub Actions).

:::

## JSON and YAML renderer

These renderer are primarily intended for automation and are to be **considered experimental**.
You can explore them using `--renderer yaml` or respective `--renderer json`.
