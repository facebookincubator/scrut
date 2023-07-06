# File Formats

Scrut supports multiple test file formats.

## File Anatomy

All test files contain one or more test cases. As mentioned before, CLIs live on a spectrum from very simple to very complicated. Reflecting that, there are two common patterns to structure test files in Scrut:
- **Coherent Test Suite** (recommended): One test file represents one use-case or behavior
- **List of Tests**: One test file contains a list of simple, not necessarily related tests

### Test Case Anatomy

Each individual test that lives in a test file is called _Test Case_ and consists of the following components:

1. A **Title**, so that a human can understand what is being done
2. A **Shell Expression**, that can be anything from a single command to a multi-line, multi-piped expression
3. **[Expectations](Expectations.md)** of the output that the Shell Expression will yield
4. Optionally the expected _Exit Code_ the Shell Expression must end in - if anything but successful execution (`0`) is expected

## Markdown Format

[Markdown](https://www.markdownguide.org/) is an amazingly simple, yet powerful language. To write _Test Cases_ in Markdown follow this guidance:

- _Shell Expressions_ and _Expectations_ live in the same code-block, that must be annotated with the language `assumption` or `scrut`
  - The first line of a _Shell Expressions_ must start with `$ ` (dollar, sign followed by a space), any subsequent with `> ` (closing angle bracket / chevron, followed by a space)
  - All other lines in the code block (including empty ones) that follow the _Shell Expression_ are considered _Expectations_
  - If an _Exit Code_ other than 0 is expected, it can be denoted in square brackets `[123]` once per _Test Case_
- The first line before the code block that is either a paragraph or a header will be used as the _Title_ of the _Test Case_

Basically like that:

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
- Code blocks that do not denote a language (or a different language than `assumption` or `scrut`) will be ignored

With that in mind, consider the following markdown file that contains not only Test Cases but arbitrary other text and other code blocks. This is idiomatic Scrut markdown files that combines tests and documentation:

````
# This is just regular markdown

It contains information an code examples that are unrelated to Scrut.

```python
import os

print("This is ignored by Scrut")
```

# It also includes Scrut tests

```scrut
$ echo Hello
Hello
```

# Embedded with other documentation

So it's a mix of test and not tests.

Any amount of tests are fine:

```assumption
$ echo World
World
```

Just make sure to write only one Test Case per code-block.
````

> **Note**: If you are testing actual markdown output, be aware that you can embed code blocks in other code blocks, if the outer code block uses one more backtick (opening and closing!) than the embedded one(s). Just have a look at the source code of this file right above this text.

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

This is up to you. The Markdown format was introduced with primarily two reasons in mind:

1. **Tests ❤️ Documentation**: The value of tests is not only in proving behavior, but also in documenting it - and thereby also in teaching it. The Markdown Test Case format allows you to keep tests around in a way that future generations of maintainers will love you for.
2. **Bad Spaces 👾**: To denote an expected empty line of output in Cram format you have to provide two empty spaces ` `. This goes counter a lot of default behavior in the development toolchain. Many CI/CD tools are tuned to automatically ignore changes that only pertain spaces. Code review tools often deliberately hide those changes. Spaces are generally hard to see in code editors - if they are visualized at all. Breaking tests that are caused by an accidentally removed or added space cause rage quitting.

If these arguments resonate with you, go for the Markdown format. If not you are probably better of with Cram that allows for a more condensed writing style. Choices, choices.
