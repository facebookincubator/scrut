# Markdown Format

We chose [Markdown](https://www.markdownguide.org/) as the primary test fle format for Scrut, because it is an amazingly simple, yet powerful language that is easily usable for humans. It is already supported by many tools and editors and it lends itself to write documentation and tests in the same location.

A markdown [test document](/docs/reference/fundamentals/test-document/) differs from a "normal" Markdown only in one way: It contains [code blocks](https://www.markdownguide.org/basic-syntax/#code) that are annotated with the `scrut` language:

````markdown showLineNumbers
# This is a normal markdown document

```scrut
$ some command
some output
```

👆 code block is a Scrut test case,
   because it is annotated with the `scrut` language.

👇 code block is NOT a Scrut test case,
   because it is not annotated with the `scrut` language.

```python
print("I am a snek")
```
````

## Test Case Anatomy

A [test case](/docs/reference/fundamentals/test-case/) in Markdown is structured as follows:

- [shell expressions](/docs/reference/fundamentals/shell-expression/) and [output expectations](/docs/reference/fundamentals/output-expectations/) live in the same code-block, that must be annotated with the language `scrut`
  - The first line of a [shell expressions](/docs/reference/fundamentals/shell-expression/) must start with `$ ` (dollar, sign followed by a space), any subsequent with `> ` (closing angle bracket / chevron, followed by a space)
  - All other lines in the code block (including empty ones) that follow the [shell expression](/docs/reference/fundamentals/shell-expression/) are considered [output expectations](/docs/reference/fundamentals/output-expectations/)
  - Lines starting with `#` that precede the [shell expression](/docs/reference/fundamentals/shell-expression/) are ignored (comments)
  - If an [exit code](/docs/reference/behavior/exit-codes/) other than `0` is expected, it can be denoted in square brackets `[123]` once per [test case](/docs/reference/fundamentals/test-case/)
- The first line before the code block that is either a paragraph or a header will be used as the *title* of the [test case](/docs/reference/fundamentals/test-case/)

Here an example:

````markdown showLineNumbers
This is the title

```scrut
$ command | \
>   other-command
expected output line
another expected output line
[123]
```
````

## Constraints

The following **constraints** apply:

- A markdown document can contain as many [test cases](/docs/reference/fundamentals/test-case/) as needed (0..n)
- Each code block in a [test case](/docs/reference/fundamentals/test-case/) may only have *one* (1) [shell expression](/docs/reference/fundamentals/shell-expression/) (each [test case](/docs/reference/fundamentals/test-case/) is considered atomic)
- Code blocks that do not denote a language (or a different language than `scrut`) will be ignored

With that in mind, consider the following markdown document that contains not only [test cases](/docs/reference/fundamentals/test-case/) but arbitrary other text and other code blocks. This is idiomatic Scrut markdown document that combines tests and documentation:

````markdown
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

Just make sure to write only one [test case](/docs/reference/fundamentals/test-case/) per code-block.
````

:::note

If you are testing actual markdown output, be aware that you can embed code blocks in other code blocks, if the outer code block uses one more backtick (opening and closing!) than the embedded one(s). Just have a look at the source code of this document right above this text.

:::

## File Suffix

Per default Scrut picks up files with `.md`, `.markdown` or `.scrut` file suffixes. This can be configured via the `--match-markdown` command line option, which accepts a glob statement like `*.md` or `*.{md,scrut}`.

## Configuration

Markdown [test documents](/docs/reference/fundamentals/test-document/) may contain inline configuration. Read more in [Reference > Fundamentals > Inline Configuration](/docs/reference/fundamentals/inline-configuration/).
