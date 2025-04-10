import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Test Maintenance

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

Tests are dynamic and require ongoing maintenance due to changes in CLI functionality or dependencies. In Scrut test documents, even if *shell expressions* remain constant, *output expectations* may need updates. Or sometimes *shell expression* must be changed which yield different output. The `scrut update` command simplifies this process by automatically updating invalid output expectations.

## Update Tests automatically

The `scrut update` command can be run on one or multiple test documents to re-execute tests and automatically update the *output expectations*.

For instance, consider the `smoke.md` document, now renamed to `version-test.md`. Previously considered a smoke test, let's treat it as a test for the CLI output for now:

````markdown title="tests/version-test.md" showLineNumbers
# Command executes successfully

```scrut
$ jq --version
jq-1.7.1
```
````

Let's say it was written in the past with an earlier version of `jq`, hence it looks like this:

````markdown title="tests/version-test.md" showLineNumbers {5}
# Command executes successfully

```scrut
$ jq --version
jq-1.7.0
```
````

Now executing this test with the `jq` version `1.7.1` will fail:

```bash title="Terminal"
$ scrut test tests/version-test.md
🔎 Found 1 test document(s)
❌ tests/version-test.md: failed 1 out of 1 testcase

// =============================================================================
// @ tests/version-test.md:4
// -----------------------------------------------------------------------------
// # Command executes successfully
// -----------------------------------------------------------------------------
// $ jq --version
// =============================================================================

1     | - jq-1.7.0
   1  | + jq-1.7.1


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
```

As expected, the version strings do not match and the Scrut test fails. Now, now can obviously straighten that out speedily with a text editor by just changing `jq-1.7.0` to `jq-1.7.1`, or by using `glob` or `regex` expectations. However, if you had tens of test files this would quickly become an annoying chore.

The `scrut update` command is a better way.

What it does is simply this:
- Execute all tests, the same way `scrut test` would
- For any that is invalid (i.e. that has output expectations that do not validate anymore) it offers you to write an updated test with fixed output expectations.

Here is how that looks in practice:

```bash title="Terminal"
$ scrut update --replace tests/version-test.md
🔎 Found 1 test document(s)
// @ tests/version-test.md:4
// -----------------------------------------------------------------------------
// # Command executes successfully
// -----------------------------------------------------------------------------
// $ jq --version
// =============================================================================

1     | - jq-1.7.0
   1  | + jq-1.7.1


? Overwrite existing document tests/version-test.md? (y/n) › no
```

The above is an interactive dialog and waits for user input whether to overwrite the existing file. If consent is given then it would overwrite `tests/version-test.md` with the output that was received from executing `jq --version`.

:::note

Useful parameters for `scrut update` are:
- `--replace` or `-r`, which writes the updated document into the same location as the original file. If not set then a file `<document-path>.new` will be created.
- `--assume-yes` or `-y`, which skips the confirmation and always assumes yes

Check out `scrut update --help` for additional parameters.

:::

:::warning

There are limits to what `scrut update` can do:
- Only `equal` and `escaped` expectations can be updated, but `glob` and `regex` would be replaced with either `equal` or `escaped`, whichever fits.
- Quantifiers (`*`, `+`, `?`) will not be considered: If an output expectation uses quantifiers and became invalid, then `scrut update` would write individual output expectations instead of one "quantified" expectation.
- Prepended and appended test documents are not updated (but you can update them individually)

:::
