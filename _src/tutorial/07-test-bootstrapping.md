import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Test Bootstrapping

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

This tutorial has produced two test documents: `smoke.md` and `expectations.md`. These documents do not extensively cover the functionality of `jq`. While this is not the primary goal of the tutorial, in real-world scenarios with complex CLIs that export numerous functions, having many test documents is common. An issue that arises from large number of documents is repetition.

As an example consider that `jq` comes with a long list of parameters. There are three that make good sense to use when optimizing the `jq` output for testing:
- `-r` (raw output): When strings are the result, quotes just make them harder to process
- `-M` (monochrome, not colored output): While that is currently the default, it may change which would break our test
- `-S` (sort keys of objects): Forcing them to be printed in a stable order yields consistent output that is easier tested

Using these make test output more stable and predictable. Let's say there are many test documents with many test cases that want those parameters set:

````markdown title="tests/boostrapped.md" showLineNumbers
# Validate many closely related things

## Validate a thing

```scrut
$ cat "$TESTDIR"/some-fixture.txt | \
>   jq -r -M -S '.some | jq(expression)'
```

## Validate another thing

```scrut
$ cat "$TESTDIR"/other-fixture.txt | \
>   jq -r -M -S '.another | jq(expression)'
```

etc
````

While having to repeat `-r -M -S` for each test is not a big problem, in real-life scenarios the parameter list can easily grow across multiple lines. Also consider typos or having to update hundreds of test cases. Hence having such repeated expressions is bothersome to write, reduces the readability of the test document, and is highly error-prone.


## Bootstrap the Environment

Mind that *shell expressions* are just that: arbitrary expressions that are interpreted and executed by a `bash` process. If a long expression would be repeatedly used manually on the command line it would not take long until someone is fed up and creates an `alias`. This is exactly what you should do in a Scrut test document:

````markdown title="boostrapped.md" showLineNumbers {3-7}
# Validate many closely related things

Setup

```scrut
$ alias jq_run='jq -r -M -S'
```

## Validate a thing

```scrut
$ cat "$TESTDIR"/some-fixture.txt | \
>   jq_run '.some | jq(expression)'
```

## Validate another thing

```scrut
$ cat "$TESTDIR"/other-fixture.txt | \
>   jq_run '.another | jq(expression)'
```

etc
````

:::note

You can use `alias` directly, because Scrut Markdown test execution already sets `shopt -s expand_aliases` for you. In Cram tests you will need to set that yourself.

Alternatively you can create a bash function or introduce an environment variable. The choice is yours.

````markdown
# Alternative Setup

```scrut
$ export JQ_RUN='jq -r -M -S'
```

# Alternative Usage

```scrut
$ cat "$TESTDIR"/other-fixture.txt | \
>   $JQ_RUN '.another | jq(expression)'
```
````

:::

The new test case that was inserted at the top is not strictly speaking a test for `jq`. However, it helps to make the test document more readable and maintainable.

## Share Setup between Test Documents

While the `alias` at the top of the document simplifies writing, reading, and maintaining tests within a single document, it can become repetitive across multiple documents. If your setup logic extends beyond a single `alias` statement, this repetition can become cumbersome.

To make the setup reusable, you can move it to a separate file. Create a file `tests/setup.sh` with the following:

```bash title="tests/setup.sh"
#!/bin/bash

alias jq_run='jq -r -M -S'
```

Now all you need to do is included that `setup.sh` file in your test document. You would do it in the same way as you would in `bash`, that is using `source` to execute and load it into the current shell process:

````markdown  title="boostrapped.md" showLineNumbers {6}
# Validate many closely related things

Setup

```scrut
$ source "$TESTDIR"/setup.sh
```

## Validate a thing

```scrut
$ cat "$TESTDIR"/some-fixture.txt | \
>   jq_run '.some | jq(expression)'
```

## Validate another thing

```scrut
$ cat "$TESTDIR"/other-fixture.txt | \
>   jq_run '.another | jq(expression)'
```

etc
````


## Prepending and Appending Test Documents

Another pattern that is related to the above is to prepend and append whole test documents. This makes especially sense if you have a large amount of shared test cases that you want to reuse across many test documents.

Scrut offers the `prepend` and `append` per-test-document configuration options for that purpose. Consider the following test document:

````markdown title="tests/setup.md"
# Setup

```scrut
$ source "$TESTDIR"/setup.sh
```
````

Here is how you can prepend all test cases in the above document (there could be more than just the one) to the test cases found in another document:

````markdown title="tests/bootstrapped.md"
---
prepend:
    - tests/setup.md
---

# Validate many closely related things

## Validate a thing

```scrut
$ cat "$TESTDIR"/some-fixture.txt | \
>   jq_run '.some | jq(expression)'
```

## Validate another thing

```scrut
$ cat "$TESTDIR"/other-fixture.txt | \
>   jq_run '.another | jq(expression)'
```

etc
````

:::note

- The paths in `prepend` and `append` must be relative to the current `TESTDIR`.
- The order matters! The files will be prepended / appended as provided.

:::

You can conceptualize these options as "test setup" (`prepend`) and "test teardown" (`append`) as seen in other testing frameworks.

### Alternative: Command-line Arguments


The `scrut test` command provides two parameters that are similar to the `prepend` and `append` configuration options:

- `--prepend-test-file-paths` / `-P`: Optional list of paths to test files which are prepended to each test file in execution.
- `--append-test-file-paths` / `-A`: Optional list of paths to test files which are appended to each test file in execution.

The difference to configuration options is that parameters are applied to *all* test documents. So if you execute `scrut test tests/` (i.e. all tests in a folder as oppose to a single test document) then the specified test document(s) will be prepended/append to *all* test documents in that folder.

Here is how that looks in practice:

```bash
$ scrut test -P tests/setup.md tests/boostrapped.md
```

:::warning

Use configuration instead of command-line parameters whenever possible, so to not break the test isolation.

:::
