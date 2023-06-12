# Test various multiline usages

Scrut Markdown supports exactly one shell expression per test. This shell expression can span multiple lines and is ultimately piped into the used shell process, so must comply with the constraints of that shell.

This test shows how multiple commands within one shell expression can be written -- assuming common Linux / MacOS shells (`bash`, `zsh`, ..) are used.

## One conjunct expression

```scrut
$ echo Foo && \
> echo Bar && \
> echo Baz
Foo
Bar
Baz
```

## Many expressions

```scrut
$ echo Foo
> echo Bar
> echo Baz
Foo
Bar
Baz
```

## Fail conjunct expression

```scrut
$ echo Foo && \
> false && \
> echo Baz
Foo
[1]
```

- **Pro**: Is evaluated as one, any error within is surfaced
- **Con**: More verbose, harder to read / write

## Fail multiple expressions

```scrut
$ echo Foo
> false
> echo Baz
Foo
Baz
```

- **Pro**: Easy to read / write
- **Con**: Hides failed executions within (only the last statement's exit code is returned)

**Note**: If you are now thinking `set -e`, then be aware that all statements in a test file are piped into the same shell process. This is intentional, so that `export` and `alias` statements can be used. It also means `set -e` in any test's shell expression that fails will immediately abort all execution and make the whole `scrut test` execution fail.

## Use-case: Shell function

Initialize shell function

```scrut
$ function bla {
>   echo BLA
> }
```

Use shell function

```scrut
$ bla
BLA
```
