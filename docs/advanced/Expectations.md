# Expectations

Expectations are predictions of one or more lines of output. _What you think a command will print out when you execute it_. My expectation when I execute `uname` is that the operating system name is printed out to the shell. On a mac, I expect the following:

```bash
$ uname
Darwin
```

> See also: [STDOUT or STDERR? What is tested](Advanced/Specifics.md#stdout-and-stderr)

The Backus-Naur form for Expectations is sweet and short:

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

The Equal Expectation is the most common one, which denotes a single line of output that ends in a newline character (`\n`). Because this expectation is so widespread, it is also the only one where you can omit the Kind. Here an example:

````
A test

```scrut
$ echo Hello
Hello
```
````

The line that consists only of `Hello` _is_ the Equal Expectation and specifies that the (first line of the) output must be equal to `Hello\n` (with `\n` being the [newline of the operating system](Advanced/Specifics.md#newline-handling)).

An extended for of the same Equal Expectation with explicit kind works as well and looks like that:

````
A test

```scrut
$ echo Hello
Hello (equal)
```
````

The explicit form makes most sense in conjunction with quantifiers:

````
A test

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

> **Note**: You can use `eq` as a shorthand for `equal`

## Equal No EOL Expectation

Very close to the above, but much rarer, the _Equal No EOL Expectation_ matches lines that do _not_ end in a newline. Consider:

````
A test

```scrut
$ echo -n Hello
Hello (no-eol)
```
````

The above `echo -n Hello` prints `Hello` _without_ a tailing newline character (there is no `\n` at the end of `Hello`).

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

````
This will work

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

> **Note**: You can use `gl` as a shorthand for `glob`

## Regex Expectation

[Regular Expressions](https://en.wikipedia.org/wiki/Regular_expression) are the most powerful, yet precise, output describing rules that are supported. That comes at the price of complexity. Explaining regular expression syntax literarily [fills books](https://www.goodreads.com/search?q=Regular+Expression), so here is not the place to attempt that. Rust uses a [RE2](https://github.com/google/re2/wiki) inspired engine. Its [syntax](https://docs.rs/regex/latest/regex/#syntax) is very similar to it. It most notably differs from Perl's [PCRE](https://en.wikipedia.org/wiki/Perl_Compatible_Regular_Expressions) because it doesn't support backtracking to ensure good performance.

Nonetheless, an obligatory example:

````
This will work

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

**Note**: All Regex Expectations have are implicitly embedded within start and end markers: `^<expression>$`. This means _regular expressions are always assumed to match the full line_. Use `.*` to explicitly match only at the end of (`.*<expression> (regex)`), or the start of (`<expression>.* (regex)`), or anywhere in (`.*<expression>.* (regex)`) a line.

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

> **Note**: You can use `re` as a shorthand for `regex`

## Escaped Expectation

CLIs usually only do (and mostly should) print out, well, printable characters. However, there are scenarios which you need to write binary data to STDOUT (e.g. consider a command line that generates a binary JPEG and pipes that output into yet another command that shrinks it or something `$ create-jpeg | shrink-image`). In addition to that adding colors can help make the output better readable - and some daredevils even throw in some emojis 🤬. Lastly, consider the good old tab character `\t`, which may be hard to read (or write) in a text editor.

Scrut tests live in Markdown or Cram files that are intended to be edited by users. They should not contain binary, non-printable data. To that end, any non-printable output can be denoted in it's hexadecimal escaped form `\xAB` (with `AB` being the hexadecimal value of the bytecode of the character) or `\t` to denote tab characters.

The following example shows an expectation of a string that renders as a bold, red font on the command line

````
Colorful fun

```scrut
$ echo -e 'Foo \033[1;31mBar\033[0m Baz'
Foo \x1b[1mBar\x1b[0m Baz (escaped)
```
````

Or consider some program that prints out two `\x00` separated strings:

````
Colorful fun

```scrut
$ some-program
foo\x00bar (escaped)
```
````

Or again, the good old tab character:

````
Love the CSV

```scrut
$ csv-generator
foo\tbar\tbaz (escaped)
```
````

> **Note**: Newlines are ignored for Escaped Expectations. So `foo\tbar (escaped)` matches both `foo\tbar\n` and `foo\tbar`.

### Examples

| Expression                | Meaning                                                                                                          |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `Hello\tWorld (escaped)`  | One output line of that starts with `Hello`, followed by a tab character, followed by `World`                    |
| `Hello\tWorld (escaped?)` | An optional output line that contains `Hello`, followed by a tab character, followed by `World`                  |
| `Hello\tWorld (escaped*)` | Any amount (0..n) of output lines that contain `Hello\tWorld`, followed by a tab character, followed by `World`  |
| `Hello\tWorld (escaped+)` | One or more (1..n) of output lines that contain `Hello\tWorld`, followed by a tab character, followed by `World` |

> **Note**: You can use `esc` as a shorthand for `escaped`

### Escaped Glob Expectations

Because it came up often enough, you can use `(escaped)` in combination with `(glob)`:

````
Glob escaped output

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

> **Note**: You can use shorthands for either. Quantifiers must be always on `glob`.
