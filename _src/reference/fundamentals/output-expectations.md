# Output Expectations

Output expectations are predictions of one or more lines of output. *What you think a command will print out when you execute it*. My expectation when I execute `uname` is that the operating system name is printed out to the shell. On a mac, I expect the following:

```bash
$ uname
Darwin
```

:::note

Understand how Scrut deals with [STDOUT and STDERR](/docs/reference/behavior/stdout-and-stderr/).

:::

The Backus-Naur form for output expectations is sweet and short:

```bnf
 <expectation> ::= <expression> | <expression> (<mod>)
  <expression> ::= TEXT
         <mod> ::= <kind> | <quantifier> | <kind><quantifier>
        <kind> ::= <equal-kind> | <no-eol-kind> | <escaped-kind> | <glob-kind> | <regex-kind>
  <equal-kind> ::= "equal" | "eq"
 <no-eol-kind> ::= "no-eol"
<escaped-kind> ::= "escaped" | "esc"
   <glob-kind> ::= "glob" | "gl"
  <regex-kind> ::= "regex" | "re"
  <quantifier> ::= "?" | "*" | "+"
```

## Quantifiers

The Quantifiers can be understood as following (nothing new if you are familiar with regular expressions):

- **`?`**: Zero or one occurrence; basically an optional output line
- **`*`**: Any amount of occurrences (`0..n`); no line, one line, more lines - all good
- **`+`**: One or more occurrences (`1..n`); at least one line, more are fine

Quantifiers can be used with most expectations, see the examples and description below for more details.

## Equal Expectation

The Equal Expectation denotes a single line of output that ends in a [newline character](/docs/reference/behavior/newline-handling/). Because this expectation is the most common one you do not need to provide the specific kind. Here an example:

````markdown showLineNumbers
# Some Test

```scrut
$ echo Hello
Hello
```
````

The line that consists only of `Hello` *is* the Equal Expectation and specifies that the (first line of the) output must be equal to `Hello\n` (with `\n` being the [newline of the operating system](/docs/reference/behavior/newline-handling/)).

An extended for of the same Equal Expectation with explicit kind works as well and looks like that:

````markdown showLineNumbers
# Some Test

```scrut
$ echo Hello
Hello (equal)
```
````

The explicit form makes most sense in conjunction with quantifiers:

````markdown showLineNumbers
# Some Test

```scrut
$ echo -e "Hello\nHello\nHello"
Hello (equal+)
```
````

### Examples

| Expression       | Meaning                                                  |
| ---------------- | -------------------------------------------------------- |
| `Hello`          | One output line of the form `Hello\n`                    |
| `Hello (equal)`  | One output line of the form `Hello\n`                    |
| `Hello (?)`      | Optional (zero or one) output line of the form `Hello\n` |
| `Hello (*)`      | Any amount (0..n) of output lines of the form `Hello\n`  |
| `Hello (+)`      | One or more (1..n) of output lines of the form `Hello\n` |
| `Hello (equal*)` | Any amount (0..n) of output lines of the form `Hello\n`  |
| `Hello (equal+)` | One or more (1..n) of output lines of the form `Hello\n` |

:::note

You can use `eq` as a shorthand for `equal`

:::

## Equal No EOL Expectation

Very close to the above, but much rarer, the *Equal No EOL Expectation* matches lines that do *not* end in a newline. Consider:

````markdown showLineNumbers
# Some Test

```scrut
$ echo -n Hello
Hello (no-eol)
```
````

The above `echo -n Hello` prints `Hello` *without* a tailing newline character (there is no `\n` at the end of `Hello`).

This Expectation could possibly only be the last line of output, so quantifiers make little sense.

### Examples

| Expression       | Meaning                                                                   |
| ---------------- | ------------------------------------------------------------------------- |
| `Hello (no-eol)` | One output line of the form `Hello` - a line that does not end in newline |

## Glob Expectation

Glob Expectations are support two wildcard characters:

- `?` matches exactly one occurrence of any character
- `*` matches arbitrary many (including zero) occurrences of any character

Together with quantifiers, this allows for powerful if imprecise matches of output lines.

````markdown showLineNumbers
# This will work

```scrut
$ echo Hello You
Hello* (glob)
```

This will work, too

```scrut
$ echo -e "Hello\nHello There\nHello World"
Hello* (glob+)
```
````

### Examples

| Expression        | Meaning                                                                 |
| ----------------- | ----------------------------------------------------------------------- |
| `Hello? (glob)`   | A single output line that starts with `Hello` followed by one character |
| `Hello* (glob)`   | A single output line that starts with `Hello`                           |
| `*Hello* (glob)`  | A single output line that contains `Hello`                              |
| `*Hello (glob)`   | A single output line that ends with `Hello`                             |
| `*Hello* (glob?)` | An optional output line that contains `Hello`                           |
| `*Hello* (glob*)` | Any amount (0..n) of output lines that contain `Hello`                  |
| `*Hello* (glob+)` | One or more (1..n) of output lines that contain `Hello`                 |

:::note

- You can use `gl` as a shorthand for `glob`.
- Escaping, like `Hello\* * (glob)`, is not supported by the [used library](https://crates.io/crates/wildmatch).

:::

## Regex Expectation

[Regular Expressions](https://en.wikipedia.org/wiki/Regular_expression) are the most powerful and precise output describing rules that are supported. That comes at the price of complexity. Explaining regular expression syntax literally [fills books](https://www.goodreads.com/search?q=Regular+Expression), so here is not the place to attempt that.

Nonetheless, an obligatory example:

````markdown showLineNumbers
# This will work

```scrut
$ echo Hello You
Hello.+ (regex)
```

This will work, too:

```scrut
$ echo -e "Hello\nEnding in Hello\nHello Start"
.*Hello.* (regex+)
```
````

:::note

All Regex Expectations are implicitly embedded within start and end markers: `^<expression>$`. This means *regular expressions are always assumed to match the full line*. Use `.*` to explicitly match only at the end of (`.*<expression> (regex)`), or the start of (`<expression>.* (regex)`), or anywhere in (`.*<expression>.* (regex)`) a line.

The `regex` Rust library that Scrut uses is an [RE2](https://github.com/google/re2/wiki) inspired engine with a very similar [syntax](https://docs.rs/regex/latest/regex/#syntax). It most notably differs from Perl's [PCRE](https://en.wikipedia.org/wiki/Perl_Compatible_Regular_Expressions) in that it doesn't support backtracking (look-around) to ensure good performance.

:::

### Examples

| Expression             | Meaning                                                                                                                                           |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Hello.* (regex)`      | A single output line that starts with `Hello`                                                                                                     |
| `.*Hello.* (regex)`    | A single output line that contains `Hello`                                                                                                        |
| `.*Hello (regex)`      | A single output line that ends with `Hello`                                                                                                       |
| `.*Hello.* (regex?)`   | An optional output line that contains `Hello`                                                                                                     |
| `.*Hello.* (regex*)`   | Any amount (0..n) of output lines that contain `Hello`                                                                                            |
| `.*Hello.* (regex+)`   | One or more (1..n) of output lines that contain `Hello`                                                                                           |
| `Foo: [0-9]+ (regex+)` | One or more (1..n) of output lines that start with `Foo` followed by a colon `:`, a whitespace ` ` and then only numbers till the end of the line |

:::note

You can use `re` as a shorthand for `regex`

:::

## Escaped Expectation

CLIs usually only do (and mostly should) print out, well, printable characters. However, there are scenarios where you need to write binary data to STDOUT. More commonly you will encounter [ANSI escape sequences](https://en.wikipedia.org/wiki/ANSI_escape_code) for color coding and so forth. Lastly, consider the good old tab character `\t`, which may be hard to read (or write) in a text editor.

Scrut tests live in text documents that are intended to be edited by users. They should not contain binary. To that end, any non-printable output can be denoted in it's hexadecimal escaped form `\xAB` (with `AB` being the hexadecimal value of the bytecode of the character) or `\t` to denote tab characters.

The following example shows an expectation of a string that renders as a bold, red font on the command line

````markdown showLineNumbers
# Colorful Fun

```scrut
$ echo -e 'Foo \033[1;31mBar\033[0m Baz'
Foo \x1b[1mBar\x1b[0m Baz (escaped)
```
````

Or consider some program that prints out two `\x00` separated strings:

````markdown showLineNumbers
# Colorful Fun

```scrut
$ some-program
foo\x00bar (escaped)
```
````

Or again, the good old tab character:

````markdown showLineNumbers
# Love the CSV

```scrut
$ csv-generator
foo\tbar\tbaz (escaped)
```
````

:::note

Newlines are ignored for Escaped Expectations. So `foo\tbar (escaped)` matches both `foo\tbar\n` and `foo\tbar`.

:::

### Examples

| Expression                | Meaning                                                                                                          |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `Hello\tWorld (escaped)`  | One output line of that starts with `Hello`, followed by a tab character, followed by `World`                    |
| `Hello\tWorld (escaped?)` | An optional output line that contains `Hello`, followed by a tab character, followed by `World`                  |
| `Hello\tWorld (escaped*)` | Any amount (0..n) of output lines that contain `Hello\tWorld`, followed by a tab character, followed by `World`  |
| `Hello\tWorld (escaped+)` | One or more (1..n) of output lines that contain `Hello\tWorld`, followed by a tab character, followed by `World` |

:::note

You can use `esc` as a shorthand for `escaped`

:::

### Escaped Glob Expectations

Because it came up often enough, you can use `(escaped)` in combination with `(glob)`:

````markdown showLineNumbers
# Glob Escaped Output

```scrut
$ csv-generator
foo\t* (escaped) (glob+)
bar\tbaz (escaped)
```
````

The above exports one or more lines of output that start with `foo` followed by tab. The last line of output is expected to be `bar`, followed by tab, followed by `baz`.

| Expression                        | Meaning                                                                                                                                |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `Hello\tWorld* (escaped) (glob)`  | One output line of that starts with `Hello`, followed by a tab character, followed by `World`, followed by anything                    |
| `Hello\tWorld* (escaped) (glob?)` | An optional output line that contains `Hello`, followed by a tab character, followed by `World`, followed by anything                  |
| `Hello\tWorld* (escaped) (glob*)` | Any amount (0..n) of output lines that contain `Hello\tWorld`, followed by a tab character, followed by `World`, followed by anything  |
| `Hello\tWorld* (escaped) (glob+)` | One or more (1..n) of output lines that contain `Hello\tWorld`, followed by a tab character, followed by `World`, followed by anything |

:::note

You can use shorthands for either. Quantifiers must be always on `glob`.

:::

## Edge-Case: Output vs Expectations

You may run into a case where you CLI output actually contains an a string that resembles an output expectation kind. For example, consider the following output:

```bash title="Output"
$ my-cli --some arg
Hello (equal)
```

The `(equal)` part above is part of the output of the CLI, not an output expectation of the kind `equal`. To account for that simply be specific with the rules. The following validates the output of the above execution:

````markdown title="tests/validate-output.md" showLineNumbers
# Some Test that accounts for output expectations in output

```scrut
$ my-cli --some arg
Hello (equal) (equal)
```
````

The string `Hello (equal) (equal)` will be read by Scrut as following:
- This is an equal output expectation, as signified by the suffix ` (equal)`
- The expected output, that precedes the "kind notation", is `Hello (equal)`
- If the output of the `my-cli --some arg` is exactly `Hello (equal)`, then the test passes

Meaning: By giving Scrut the explicit ` (equal)` suffix, it will be able to distinguish between the output expectation and the output itself.
