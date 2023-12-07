---
sidebar_position: 2
---


# Tutorial

A walkthrough of Scrut use from start to end. For in-depth information: follow the ~~white rabbit~~ inline links.

> The beginning is perhaps more difficult than anything else, but keep heart, it will turn out all right. - _Vincent van Gogh_

This guide is written with the following target audiences in mind:

1. **CLI owners / contributors**, that care about the quality of a specific CLI and therefore want to
   - Prove the behavior of the CLI in the form of integration / end-to-end tests tests
   - Document the CLI behavior for themselves of future developers of the CLI
2. **System administrators / operators**, that care about the CLI tools they work with and need to
   - Establish understanding and verify assumptions about their CLI tools
   - Document behavior of their CLI tools for themselves or future generations

## Prerequisites

To make it very simple to follow along, this guide uses the modern, but well established [`jq`](https://github.com/stedolan/jq) command line tool as the CLI that is tested in all provided code examples. Deep understanding of `jq` is not required, but it would help if you have at least some grasp what it does and how to use it. If that is not the case, yet: it is a truly, amazingly useful tool; now is a great time to learn about!

The following should work on your terminal:

```sh
# scrut itself should be installed
$ scrut --version
scrut 0.2.0

# jq should be installed
$ jq --version
jq-1.6
```

> **Note**: In all shell code blocks within this document lines prefixed with `$ ` are commands, lines prefixed with `# ` are comments and any other line can be assumed to be the output of the previous command

### About file structure

Scrut does not require any particular file structure. This tutorial is assuming that the files would be stored in a `integration-tests` subdirectory together with the source-code of the CLI that is tested.

```bash
# going to the directory that contains the source code
$ cd ~/Projects/jq

# creating a new directory that is going to contain the tests
$ mkdir integration-tests
```

Although Scrut has no requirements towards file structure it is recommended, that all test relating files (see more below) are in the same directory as the test files themselves, which makes referencing them easier.

## Decide what to test first

What then is the first thing to test about our CLI `jq`? What is the first thing to test about any CLI? Maybe you have a great answer that fits perfectly for your specific CLI. If you don't then consider to start with a [smoke test](<https://en.wikipedia.org/wiki/Smoke_testing_(software)>): _When I switch it on, do I see smoke rising up?_

Translated to a CLI that means: executing the tool in the most basic way possible, does it panic / fatal / die unexpectedly? Considering you keep developing your CLI, such a basic test answers the question: _Did you break something very fundamental?_

And what would be a good smoke test for a CLI? For `jq` it is the execution from above (`jq --version`) seems like a great candidate. For other CLIs it might be `--help` instead. Either way, you want to choose something that doesn't have much complexity, that doesn't rely on any external dependencies. If you are the author of the CLI that should be easy to find.

## Pattern: Automatic Test Creation

Finally, let's get to writing the test. Actually writing seems too bothersome. Sure, you could, but how about you generate it instead? Do that:

```bash
$ scrut create --output integration-tests/smoke.md -- jq --version
Writing generated test to `integration-tests/smoke.md`
```

Ok, let me unpack that for you:

- `scrut create` - tells Scrut to execute a command and create a test from its output
- `--output integration-tests/smoke.md` - lets `scrut` know where to write the created test to
- `--` - signifies the end of options for scrut; all that follows is part of the command for which a test is generated
- `jq --version` - that is the command (the _Shell Expression_) which `scrut` is going to execute and from which's output it is going to generate test _Expectations_

This also could have been written differently:

```bash
$ echo "jq --version" | scrut create - > integration-tests/smoke.md
Writing generated test to STDOUT
```

Here the string `jq --version` was piped to the STDIN of `scrut create` (which was made aware of that by having one argument `-`) and the output (to STDOUT) was delegated into the same output file as before.

Both are valid forms and result in the same outcome, that is a new test in the file `integration-tests/smoke.md`. The contents of that file should be like that (aside from the version string, that is likely different for you):

````markdown
# Command executes successfully

```scrut
$ jq --version
jq-1.6
```
````

While you are looking at it, how about you change that title to `Smoke test` or something like that. **Half of the value of a Scrut test file is the documentation, so it is always worth to put in some time to clarify intentions and describe expectations**.

Don't touch the rest - for now. We'll get to that in a minute. You can read up on the [anatomy of the file](advanced/file-formats.md#file-anatomy), here a very quick primer:

- Scrut test files are [markdown documents](advanced/file-formats.md#markdown-format)
- Code blocks of language `scrut` contain the tested commands and the expected output

### Run the first Test

Running tests is the bread and butter of Scrut. It is - literally - what it is made for. So without further ado:

```bash
$ scrut test integration-tests/smoke.md
Validation succeeded
```

Nice! That works. As it should be, since Scrut create the test for you. Although that was a bit anticlimactic. Let's make it more fun and go break it 🤡. Change the contents of the file like so:

````markdown
# Smoke test

```scrut
$ jq --version
foo
```
````

Now run it again:

```bash
$ scrut test integration-tests/smoke.md
// =============================================================================
// @ integration-tests/smoke.md
// -----------------------------------------------------------------------------
// # Smoke test
// -----------------------------------------------------------------------------
// $ jq --version
// =============================================================================

   1  | - foo
1     | + jq-1.6
```

Ok, it is getting interesting. What you are seeing here (likely in color) is an output validation error. The output expectations in the test file do not match with the output the command actually spits out. This is how you read it:

- `@ integration-tests/smoke.md`: Location of the test file
- `# Smoke test`: Title of the test in the file
- `$ task --version`: Shell expression that resulted in invalid output
- ```
     1  | - foo
  1     | + jq-1.6
  ```
  The first line `1 | - foo` denotes that `foo` was expected from the test, but is missing in the output. The next line `1 | + jq-1.6` denotes that `jq-1.6` was printed out as 1st line from the command, but is missing in the test.

## Pattern: Resilient Tests

This is actually a good point in time to speak about brittle tests. Having the version (here `jq-1.6`) in the `smoke.md` file is not a good idea. Why? Because it is likely to change, because you keep developing it. Or someone is. Having that string in the test file will just create the worst kind of all work down the line: toil.

Also consider: Does having the version in there really provide value? The idea of the smoke test is to fail if things are so broken, that basically nothing works anymore. From that perspective, there is no need to check about the version: let's get rid of this nascent technical debt.

So how do you do that? Well, how would you do it on the shell? You would do something like that:

```sh
$ jq --version > /dev/null
```

And that is exactly how you would do it in the test:

````markdown
# Smoke test

```scrut
$ jq --version > /dev/null
```
````

Is that still a meaningful test? Yes, it is! It still tests whether the command executes successfully. What does successfully mean? Well, whether it exits with a `0` exit code. _That is an implicit test any test case will automatically provide_. Don't take my word for it, though. Change the expected exit code to, say, `10` and see what happens. Just add a new line containing `[10]` after the shell expression:

````markdown
# Smoke test

```scrut
$ jq --version > /dev/null
[10]
```
````

Now test it:

```bash
$ scrut test integration-tests/smoke.md
// =============================================================================
// @ integration-tests/smoke.md
// -----------------------------------------------------------------------------
// # Smoke test
// -----------------------------------------------------------------------------
// $ jq --version > /dev/null
// =============================================================================

unexpected exit code
  expected: 10
  actual:   0

## STDOUT
## STDERR
```

As promised: it fails. The output should be self explanatory. Read more about [exit codes here](advanced/specifics.md#exit-codes).

Going forward remove the `[10]` again, so that the test is in a working state.

## Pattern: Test Fixtures

Ok, let's start with testing actual functionality. No worries, we won't attempt to cover all that `jq` can do with tests in this tutorial. Just enough to show some good to know patterns. Here is one, if a bit obvious: a good idea to start with any test is executing it on the shell.

Since `jq` is a neat tool to manipulate JSON, we need some JSON to manipulate. Let's use the [same as the `jq` tutorial itself](https://stedolan.github.io/jq/tutorial/), that is the Github history of [the `jq` repository](https://github.com/stedolan/jq):

```sh
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5'
# not gonna show the output, it is a lot
```

Let's say we want to write a test that proves and documents the (imho) core functionality of `jq`: mutating JSON. As an example we are going to reduce those huge JSON dumps into something more manageable: _who's commit was committed when_. Each result item should have the following form: `{"who": "<name>", "when": "<date>"} `. This is how you can achieve that on the the command line (names changed):

```sh
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' | \
  jq '[.[] | {who: .commit.author.name, when: .commit.committer.date}]'
[
  {
    "who": "Person Name",
    "when": "2022-05-26T21:04:32Z"
  },
  {
    "who": "Another Person",
    "when": "2022-05-26T21:02:50Z"
  },
  {
    "who": "Even More",
    "when": "2022-05-26T21:02:10Z"
  },
  {
    "who": "And so forth",
    "when": "2022-05-26T21:01:25Z"
  },
  {
    "who": "Name Name",
    "when": "2022-05-26T20:53:59Z"
  }
]
```

Ok, that shows that the transformation of the output works as we assumed it would. However, you probably have noted, using the `curl` output in the a test will not be very resilient, as the output is prone to change.

Since we are not really interested in the functionality of `curl` or Github (and quite frankly could without network dependencies), let's instead store the current output of the `curl` execution into a _test fixture file_ in our `integration-tests` folder. This way we have a consistent input to run our test on:

```sh
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' > integration-tests/commits.json
```

Now we can start with writing the actual test file. Instead of using `scrut create`, start with the following template in `integration-tests/transform-input.md`:

````markdown
# Transform input

```scrut
$ cat "$TESTDIR/commits.json" | \
> jq '[.[] | {who: .commit.author.name, when: .commit.committer.date}]'
[
  {
    "who": "Person Name",
    "when": "2022-05-26T21:04:32Z"
  },
  {
    "who": "Another Person",
    "when": "2022-05-26T21:02:50Z"
  },
  {
    "who": "Even More",
    "when": "2022-05-26T21:02:10Z"
  },
  {
    "who": "And so forth",
    "when": "2022-05-26T21:01:25Z"
  },
  {
    "who": "Name Name",
    "when": "2022-05-26T20:53:59Z"
  }
]
```
````

> **Note**: The second (and any subsequent) line of a command starts with a `>` character - unlike the first, which starts with a `$` ([read more](advanced/file-formats.md#markdown-format)). The tailing `\\` in the first command line is needed, because `/bin/bash` needs it (both lines, stripped by their starting `$` or `>` character, are ultimately passed to the shell process, hence must comply with it's requirements).

### Tests directory isolation

You may have noted the that the `commits.json` file is referred to as `"$TESTDIR/commits.json"`. The reason for that is that each test is executed from within an empty test directory. The absolute path to the directory, where the actual test file is in is available via the `$TESTDIR` environment variable. Since `commits.json` is located in the same directory as `transform-input.md` the expression `"$TESTDIR/commits.json"` contains the absolute path to the `commits.json` file ([read more](advanced/specifics.md#test-isolation)).

## Pattern: Test Bootstrapping

There is one more thing that should be done to make the test resilient: `jq` has a couple of command line parameters that decide how the output is being rendered. There are two in particular, which should be set in our case:

- `-r` (raw output): Pertains to non-JSON output, in which strings would be quoted without it (let's not - easier to pipe into other command line programs)
- `-M` (monochrome, not colored output): While that is currently the default, it may change which would break our test
- `-S` (sort keys of objects): Currently, the keys are outputted as we provided them - but to be safe (have a resilient test), lets just explicitly sort them, then there is no question in their order

Using both of those keys would change the command in the `jq <..>` command in the test to `jq -r -M -S <..>`.

Thinking ahead, we are going to use these flags in _every test_, for the same reason why we are using it here (be very sure about the expected output). With that in mind, consider the following bash script:

```bash
#/bin/bash

# tell bash exporting aliases is fine
shopt -s expand_aliases

# alias `jq`, so that it always executes with the two parameters
alias jq='jq -r -M -S'
```

Store the above file under `integration-tests/setup.sh`, and then we can make use of it in our test file:

````markdown
# Test transformation

Test whether `jq` transforms tests as we

## Bootstrap

```
$ source "$TESTDIR/setup.sh"
```

## Transform input

```scrut
$ cat "$TESTDIR"/commits.json | \
> jq '[.[] | {who: .commit.author.name, when: .commit.committer.date}]'
[
  {
    "when": "2022-05-26T21:04:32Z",
    "who": "Person Name"
  },
  {
    "when": "2022-05-26T21:02:50Z",
    "who": "Another Person"
  },
  {
    "when": "2022-05-26T21:02:10Z",
    "who": "Even More"
  },
  {
    "when": "2022-05-26T21:01:25Z",
    "who": "And so forth"
  },
  {
    "when": "2022-05-26T20:53:59Z",
    "who": "Name Name"
  }
]
```
````

> **Note**: The order of `who` and `when` changed due to `-S`.

As you can see there are now two code blocks of the type `scrut` in the same file. That means there are two tests in that one file. This is fine, you can have [as many test as make sense to you in a file](advanced/file-formats.md#file-anatomy). Scrut [executes them in order](advanced/specifics.md#sequential-or-parallel-execution), within the same shell process, which allows the `alias jq=..` set in `setup.sh` to affect the `jq` execution in the test file.

**Bootstrapping tests is a very common strategy in Scrut** and is considered idiomatic.

### Bootstrapping, sounds familiar?

If you are familiar with unit testing (in whatever language), you likely came across the [test suite pattern](https://en.wikipedia.org/wiki/Test_suite). If not, then in (very) short: A test suite is a semantic cohesive collection of tests, which is often run against different implementations of the same interface. Imagine a storage backend interface, for which an implementation `LocalStorage` writes on a local disk and `RemoteStorage` writes somewhere in the cloud. Both implement the same `Storage` interface and therefore can be tested by the same test suite `StorageTestSuite`.

In those scenarios it is not uncommon that each test-suite run executes specific "setup code" for each implementation, before all the tests are executed. You may often find methods named like `setupTests`, `beforeTests` or something akin.

**A variant of bootstrapping is seeding** where a specific methods are executed once before _each_ test (as oppose to: once before _all_ tests). Although the terms may be also be used interchangeable (depends on the language of the testing framework and developer's choice). Translated to Scrut you could have `seed-some-state.sh` files, that are then included in one or multiple tests, to keep the tests themselves clean and the code d.r.y.

## Pattern: Update as a Workflow

In the previous section quite a lot of copying from the terminal into text files happened. A tad bothersome and smells like a bad tedious process. Indeed. There is a better way.

Let's start with a new test. `jq` has [a lot of built-in functions](https://stedolan.github.io/jq/manual/#Builtinoperatorsandfunctions), so there is plenty to pick from. Since we were already interested in that committer date earlier, lets write a test for the `fromdate` function. Start with the following template, which is basically a copy of the previous test, but with the new command we want and with all outputs striped:

````markdown
# Test built-in `fromdate`

Assure the `fromdate` function parses ISO 8601 dates into unix timestamps

## Bootstrap

```scrut
$ source "$TESTDIR/setup.sh"
```

## Use `fromdate`

```scrut
$ cat "$TESTDIR"/commits.json | \
> jq '.[] | .commit.committer.date | fromdate'
```
````

**Having clear intentions in the leading markdown of a test file is a good practice**. Here it makes it clear that we are expecting the output of some unix timestamps. Since we don't have any, it is to be expected that the test execution will fail. Only one way to be sure:

```bash
$ scrut test integration-tests/builtin-fromdate.md
// =============================================================================
// @ integration-tests/builtin-fromdate.md
// -----------------------------------------------------------------------------
// # Use `fromdate`
// -----------------------------------------------------------------------------
// $ cat "$TESTDIR"/commits.json | \
//   jq '.[] | .commit.committer.date | fromdate'
// =============================================================================

1     | + 1653599072
2     | + 1653598970
3     | + 1653598930
4     | + 1653598885
5     | + 1653598439
```

This output tells us two things:

1. It seems `fromdate` can parse our dates and transform them into unix timestamps
2. The test fails, because it does not mention the expected output

At least the latter is not completely surprising. In order to make the test green, we could again copy the output into the test. However, there is a better way - as promised:

```bash
$ scrut update --replace integration-tests/builtin-fromdate.md
```

This shows you the same failed test output again. However, in addition it ends in a prompt that asks you whether the test file should be overwritten:

```bash
> Overwrite existing file `integration-tests/builtin-fromdate.md`?
```

Hit `y` here, which will cause `scrut` to update your test and add the missing output lines after the command for you.

**Writing tests and using update to fill in the outputs is good practice** for creating new tests and also for maintain existing ones: Imagine you fix a typo in the command output. Run `scrut update <file>` to fix the test. Does the typo change a lot of tests? Run `scrut update <directory>` and be done.

## Powerful Expectations

Take a step back and consider the test cases we wrote so far - and compare them against real-live scenarios. One thing may peak out you: Using a the `commits.json` file as a test fixture is a neat way to assure that we always work on the same input data. However, especially in the end-2-end testing space, things are not always possible. Things are not as neat and tidy.

Leave the idea of testing the functionality of `jq` for a moment behind, so you can think about writing tests for situations where the data your tests run on is outside of your control.

Let's revisit our `transform-input.md` test file from before. Copy it into `transform-input-live.md` and change in that new file the command into the following:

````markdown
```scrut
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' | \
> jq '.[] | .commit.author.name + ";" + .commit.committer.date'
```
````

This means: we are back to using the live data (to simulate "dirty" / unpredictable data). Also the output is no longer JSON, but a single line string per commit with the format `<name>;<date>`.

First, run `scrut update` on it and overwrite the contents. The modified `transform-input-live.md` file should look something like that (with different names and dates):

````markdown
# Test transformation

## Bootstrap

```scrut
$ source "$TESTDIR/setup.sh"
```

## Transform input from live data

```scrut
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' | \
> jq '.[] | .commit.author.name + ";" + .commit.committer.date'
Person Name;2022-05-26T21:04:32Z
Another Person;2022-05-26T21:02:50Z
Even More;2022-05-26T21:02:10Z
And so forth;2022-05-26T21:01:25Z
Name Name;2022-05-26T20:53:59Z
```
````

We already established, that having this specific content in there is brittle and will cause headache down the line. So where is this going?

At this point it becomes necessary to understand that each of the output lines in the test are actually [output expectations](advanced/expectations.md). The last line of the above output could also be written as:

```
Name Name;2022-05-26T20:53:59Z (equal)
```

The tailing ` (equal)` is the _type_, telling Scrut that this is, well, an expectation which should match exactly the provided expression (like the `==` equal operator). Since those are the most common ones, and it is so much more readable to _not_ have `(equal)` everywhere, you can omit it. However, this the only expectation that allows you to omit the type.

### Glob

Scrut has two expectation types that would work here. Lets start with simpler one, that is powerful, but not very precise, though easy to write and read. It is the [glob expectation](advanced/expectations.md#glob-expectation). Consider the following:

````markdown
## Transform input from live data

```scrut
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' | \
> jq '.[] | .commit.author.name + ";" + .commit.committer.date'
*;20*Z (glob)
*;20*Z (glob)
*;20*Z (glob)
*;20*Z (glob)
*;20*Z (glob)
```
````

Without going [into full detail](advanced/expectations.md#glob-expectation), glob supports two wildcard characters `*` for any amount of any character and `?` for a single arbitrary character. Each of the above expectations translates to:

- Any string that is followed by `;20`
- Followed by anything
- Ending in `Z`

> **Note**: _anything_ means anything _but_ a newline character

Using the glob expectation like this should cover about any possible output - at least until the year 2100. There should be little maintenance in the short- to midterm. That is reasonable resilient - but rather imprecise.

On that note: As you can see, we repeated the same expectation five times. **Each line of output must have a matching expectation or the test fails**. That also means: Having exactly five expectations is a test in itself, which would fail for zero or four or six lines of outputs equally.

### Regular expression

The above headline bestows fear in many and delight in some. So it is up to you to read this paragraph or skip it entirely. If you are not familiar with regular expressions, maybe you take this as an opportunity to learn about them - although this is way beyond the scope of this how-to.

Lets jump right into it then: `scrut` supports [regular expression expectations](advanced/expectations.md#regex-expectation) with the ` (regex)` type. Rewriting the test from above could look like that (well, one variant):

````markdown
## Transform input from live data

```scrut
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' | \
> jq '.[] | .commit.author.name + ";" + .commit.committer.date'
\w+(?:\s\w+)*;\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}Z (regex)
\w+(?:\s\w+)*;\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}Z (regex)
\w+(?:\s\w+)*;\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}Z (regex)
\w+(?:\s\w+)*;\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}Z (regex)
\w+(?:\s+\w+)*;\d{4}\-\d{2}\-\d{2}T\d{2}:\d{2}:\d{2}Z (regex)
```
````

This is much more precise than the above glob expectation - at the cost of readability. There is room for error, that likely won't capture all possible name writings (e.g. `Forename M. Surname` would fail) - feel free to optimize.

### Quantifiers

A last, but extremely useful feature - especially when testing multiple lines of similar formed output - are _Quantifiers_.

Consider the `curl` query from above. It ends in `?per_page=5`, which indicates that we should expect _up to_ five items - could be less, though. A different valid scenario would be too much output. Imagine your CLI outputs, say, hundreds or even thousands of lines. That would make any test file unreadable, aka unmaintainable, for humans. A test that cannot be understood is equal to no test - maybe even worse.

So how would a test look that addresses those issues? Especially when knowing that _every output line_ must be covered by an expectation? Enter the _expectation quantifier_, which allows you to define quantities for expectations. Consider this:

````markdown
## Transform input from live data

```scrut
$ curl 'https://api.github.com/repos/stedolan/jq/commits?per_page=5' | \
> jq '.[] | .commit.author.name + ";" + .commit.committer.date'
*;20*Z (glob+)
```
````

Note the `+` symbol after the `glob` word. That is a quantifier. [Read more about them here](advanced/expectations.md#quantifiers). Suffice to say that there are three (`?` = optional, `*` = 0 or more, `+` = 1 or more). Meaning, this single line covers all the possible output lines that match this form.

## Pattern: Structure by use-case

This tutorial already talked about how to structure tests inside a file (having bootstrapping at the top, followed by the actual tests). As a last topic let's talk for a minute about how to structure test files (within folders).

As noted at the start of this document, Scrut can be very useful for CLI owners and system administrators alike. The former may concentrate on testing and documenting a single CLI. The latter may concentrate on testing and documenting the interplay of multiple command line tools at once, maybe the process of a runbook, or a specific operation to recover a database or something like that.

Either way **it is good practice to isolate every use-case into a single file**. That could be one test file per sub-command of the CLI that is tested or one test file per runbook that is tested. Whatever makes most sense. The purpose should be to gain the most information possible out of a failing test: _Test `A.md` is failing, but test `B.md` is not, that indicates that `feature X` is broken_.

For `jq` that could mean to write a single [file per function](https://stedolan.github.io/jq/manual/#Builtinoperatorsandfunctions) `jq` exposes. However, if `jq` already has a unittest suite that covers each function, maybe it makes more sense to concentrate on testing [I/O](https://stedolan.github.io/jq/manual/#IO) and also maybe whether [modules](https://stedolan.github.io/jq/manual/#Modules) work as expected.

## Next steps

You did it. You are a _scrutacean_ now ([rust](https://www.rust-lang.org/) developers are called _rustaceans_, scrut is build in rust, there you go). If you want, go ahead and write some additional tests for `jq`, or dig deeper into the rest of [documentation](advanced/).
