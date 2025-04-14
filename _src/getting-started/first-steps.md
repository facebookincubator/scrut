---
sidebar_position: 2
---

import OssNote from '../fb/components/_oss-note.md';

# First Steps

<FbInternalOnly><OssNote /></FbInternalOnly>

Now that you have a [`scrut` binary installed](/docs/getting-started/installation) the fun can begin.

## Create your first test

Create a new directory you want to work in.
Then create a markdown file named `getting-started.md` in that directory with the following contents:

````markdown title="getting-started.md" showLineNumbers
# Get Testing

All code blocks marked with `scrut` are tests.

## Hello World

```scrut
$ echo Hello World
Hello World
```

This will work!

## Sad World

```scrut
$ echo Sad World
Hello World
```

This will fail!

## Ignore World

```other
$ echo Other World
Hello World
```

This is ignored.
````

Don't worry about the contents for now. All will be explained later.
Open a terminal in the directory where you created the markdown file and run a test:

```bash title="Terminal"
$ scrut test getting-started.md
🔎 Found 1 test document(s)
❌ getting-started.md: failed 1 out of 2 testcases

// =============================================================================
// @ getting-started.md:17
// -----------------------------------------------------------------------------
// # Sad World
// -----------------------------------------------------------------------------
// $ echo Sad World
// =============================================================================

1     | - Hello World
   1  | + Sad World


Result: 1 document(s) with 2 testcase(s): 1 succeeded, 1 failed and 0 skipped
```

It failed. Great! That is expected.

## What just happened?

You ran your first test. Let's walk through it, starting with the command you executed:

```bash
$ scrut test getting-started.md
```

This line should be self-explanatory, but let's be explicit:

- `scrut` is your previsouly installed Scrut binary
- `test` is the subcommand that tells Scrut to run tests
- `getting-started.md` is the path to the file you just created, telling Scrut to run the tests defined within

```bash
🔎 Found 1 test document(s)
```

Scrut reports that it found one *test document*.

:::info

Scrut works with files that contains tests. That makes the word *test* ambiguous: Does that refer to the file? Or to a test within the file? To make it clear what is mean Scrut uses **document** or **test document** to refer to the file that contains tests and **test case** to refer to a test within a test document.

:::

```bash
❌ getting-started.md: failed 1 out of 2 testcases
```

This line points to a test document (`getting-started.md`) and lets you know that one of the two *test cases* in that file have failed.


Next up is the output that starts with `//`, which you will see whenever there is a failure in a testcase:

```bash
// @ getting-started.md:17
```

An error occured in the command that can be found in line 17 of the `getting-started.md` file.


```bash
// # Sad World
```

The failing test title (i.e. the headline preceeding the code block, see the `getting-started.md` file above).

```bash
// $ echo Sad World
```

The *shell expression* that was executed and ended in failure.

:::info

Scrut calls the commands that are being tested **shell expressions**. Each test contains a single shell expression. It may span across multiple lines. More about that later.

:::


```bash
1     | - Hello World
```

The expected output, as defined in `getting-started.md` below the test command.
The `1` signifies that this is the first expected output line, measured after the command that is tested (`echo Sad World`).

```bash
   1  | + Sad World
```

The actual output, that did not match the expection.
The `1` signfies that this is the first output line, measured from the output of the command that is tested (`echo Sad World`).


```bash
Result: 1 document(s) with 2 testcase(s): 2 succeeded, 0 failed and 0 skipped
```

The test file contained 2 tests, one succeeded and one failed.
Note that only the code blocks marked with `scrut` were considered.
The last code block in the `getting-started.md` file that has the "langauge" `other` set was (intentionally) ignored.

**In summary:**
- You told Scrut to run tests in a file (`getting-started.md`)
- One of the tests in that file failed
- Scrut told you which test failed, where in the file it is and how it did not match the expectations that are defined in the file

## Fix it!

To fix the test you need to understand a bit more about the syntax in the `getting-started.md` file.
Have a look at the second code block in the file that reads:

````markdown
```scrut
$ echo Sad World
Hello World
```
````

The first line here that starts with a `$` (dollar) sign is, as mentioned before, the *shell expression* of the *testcase*.
The next line is what validates the output of the execution. Scrut calls this a **output expectation**. More on that later.

You can read the test as following:
*When I execute `echo Sad World`, then I expect to see `Hello World` printed.*

Obviously that expectation is wrong: The expected output for `echo Sad World` is `Sad World`, not `Hello World`.

So in order to fix this simple test, all you need to do is align either (not both!) of the following:
- Use `$ echo Hello World` as the shell expression.
- Use `Sad World` as the test expectation.

That means either of the following tests are valid:

````markdown
```scrut
$ echo Hello World
Hello World
```
````

or

````markdown
```scrut
$ echo Sad World
Sad World
```
````

Once you have "fixed" the test subsequent `scrut test` execcutions will succeed:

```bash title="Terminal"
$ scrut test getting-started.md
🔎 Found 1 test document(s)

Result: 1 document(s) with 2 testcase(s): 2 succeeded, 0 failed and 0 skipped
```

:::info

Scrut by default only prints tests that did not succeed. The `Result:` line is always printed and shows you how many documents processed, but if you want to see the tests as they are being processed use the `--verbose` flag:

```bash title="Terminal"
$ scrut test --verbose getting-started.md
🔎 Found 1 test document(s)
✅ /tmp/getting-started.md: passed 2 testcases

Result: 1 document(s) with 2 testcase(s): 2 succeeded, 0 failed and 0 skipped
```

:::

## Next Up

Congratulations! You have taken the first step, which is always the hardest.

Go ahead and start with the [tutorial](/docs/tutorial).
