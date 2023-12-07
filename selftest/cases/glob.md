# Glob Expectations

Scrut `(glob)` expectations support wildcard matches using `*` for any amount of any character and `?` for one arbitrary character.

This test file show-cases the use.

## Glob one character

```scrut
$ echo -e 'foo\nfun'
f?? (glob+)
```

## Glob any amount of characters

```scrut
$ echo -e 'foo\nfun\nfable'
f* (glob+)
```

## Glob in JSON output

```testcase
$ echo '{"value":"foo: {\"bar\":333}"}'
{"value":"foo: {\"bar\":*}"} (glob)
```

## Glob with escaped characters

```scrut
$ echo -e 'foo \033[1mbar\033[0m baz'
f?? \x1b[1mbar\x1b[0m baz (escaped) (glob)
```
