import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Test Environment

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

In the previous chapter a test was created that validates the output of a `jq` expression with input from the Github API. Using `glob` output expectations made the test stable, but at the cost of losing precision. This chapter explains how both stability and precision can be achieved.

## Test Fixtures

The reason why `glob` was used was that the output of `curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5'` was simply not stable: The output is changing over time.

However, there is another alternative to stabilize the test: Storing the output of the `curl` command in a file and using the file contents as input for `jq`. A classic **test fixture**. This way the test input will not change, making the test stable, while retaining the higher precision.

First create a new file `expectations-fixture.txt` in **the same directory** as `expectations.md` and add the following:

```plain title="tests/expectations-fixture.txt" showLineNumbers
2025-03-28T00:57:51Z,dependabot[bot]
2025-03-28T00:56:51Z,dependabot[bot]
2025-03-28T00:55:39Z,dependabot[bot]
2025-03-27T23:43:06Z,itchyny
2025-03-27T23:42:44Z,itchyny
```

Now with that in place, the `expectations.md` test document can be updated to use the fixture instead of the `curl` command:

````markdown title="tests/expectations.md" showLineNumbers {4-5}
# Output Expectations

```scrut
$ cat "$TESTDIR"/expectations-fixture.txt | \
>   jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
2025-03-28T00:57:51Z,dependabot[bot]
2025-03-28T00:56:51Z,dependabot[bot]
2025-03-28T00:55:39Z,dependabot[bot]
2025-03-27T23:43:06Z,itchyny
2025-03-27T23:42:44Z,itchyny
```
````

:::note

Test fixtures are extremely helpful to increase test isolation. Using the fixture above categorically removed internet access, `github.com` availability and `curl` availability and functionality as dependencies from the test.

:::

## Environment Variables

The `TESTDIR` environment variable that is now used in `expectations.md` is a special variable that is automatically set by Scrut. It contains the path of the directory where the test file that is currently being executed is located.

Two other very useful environment variables are:
- `TMPDIR`: A temporary directory that is created for every Scrut execution automatically. It is removed when all tests are done.
- `SCRUT_TEST`: Is set to contain the path and the line number where the where the test case is located in the test document (.e.g `dir/test.md:123`)

:::info

Learn about all [environment variables](/docs/reference/fundamentals/environment-variables/) that Scrut maintains.

:::

## Working Directory

Scrut creates a temporary directory for each test document that is processed in `scrut test` or `scrut update`. This directory becomes the current working directory (CWD) for the test execution.

This working directory directory does not contain test documents or test fixtures. This is the reason why the environment variable `TESTDIR` was earlier used to `cat` the fixture file:

````markdown
```scrut
$ cat "$TESTDIR"/expectations-fixture.txt | \
>   jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
-- %< --
```
````

While the working directory is temporary, it is still shared between all test cases. So files can be written into it and picked up by later test cases. However, the above mentioned `TMPDIR` environment variable is even better for that.

To access the current directory either execute `pwd` or access the `PWD` environment variable:

````markdown
```scrut
$ echo "I am in $(pwd), which is not $TESTDIR"
```
````

:::info

Learn more about the working directory in [Reference > Behavior > Working Directory](/docs/reference/behavior/working-directory/).

:::
