import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Basic Expectations

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

The smoke test from the previous chapter validates that executing `jq --version` will output the string `jq-1.7`. While this is a good start, it also has a few problems:

- It is not really a smoke test, because it tests more than "does it blow up?"
- It will fail down the line when `jq` is being updated (as `jq` is only a stand-in for the CLI you are testing an developing, you will likely have constant version upgrades to deal with)

To make this a proper smoke test it needs to shed the validation of the specific version and only validate that the test execution does not "blow up".

## Ignore Command Output

Consider how you would get rid of the output when executing the command `jq --version` on the shell. You would likely do something this:

```bash title="Terminal"
$ jq --version > /dev/null
```

:::info

The suffix `> /dev/null` redirects the output that `jq --version` writes to **STDOUT** to `/dev/null`, resulting in nothing being written to STDOUT. As STDERR is not considered by default this is sufficient.

:::

And this is exactly what needs to be changed in the test document:

````markdown title="tests/test.md" {4}
# Command executes successfully

```scrut
$ jq --version > /dev/null
```
````

With the *output expectation* removed this test will do as a smoke test.

## Exit Code Validation by Default

Scrut automatically validates that the exit code of the execution of the *shell expression* is `0` which signifies that the execution ended without any failure. If it is not `0`, then the execution is considered a failure and the validation of the *test case* will fail.

That means: We are already testing if it does "blow up", as Scrut would fail the test if the execution blows up and ends in a non-zero exit code.

To make this clear, consider the following document of a test that will fail:

````markdown title="tests/fail.md"
# Test will fail

```scrut
$ false
```
````

:::info

The `false` command always fails and exits with a the exit code `1`.

:::

And here is how Scrut would tell you about the failure:

```bash title="Terminal"
$ scrut test tests/fail.md
🔎 Found 1 test document(s)
❌ tests/fail.md: failed 1 out of 1 testcase

// =============================================================================
// @ tests/fail.md:4
// -----------------------------------------------------------------------------
// # Test will fail
// -----------------------------------------------------------------------------
// $ false
// =============================================================================

unexpected exit code
  expected: 0
  actual:   1

## STDOUT
## STDERR


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
```

## Expect a Non-Zero Exit Code

If the *shell expression* that is being tested is actually expected to return a non-zero exit code, then the `[<exit-code>]` expectation can be used to communicate this to Scrut. Here an example:

````markdown title="fail.md"
# Test will fail

```scrut
$ false
[1]
```
````

The `[1]` signifies that the test validation should expect an exit code of `1`. Now the above document is valid again:

```bash title="Terminal"
$ scrut test tests/fail.md
🔎 Found 1 test document(s)

Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

If any different number than `1` would have been set then the validation would fail.

:::note

Scrut automatically assumes `0` exit code by default. Specifying it with `[0]` is not needed (but also not invalid).

:::
