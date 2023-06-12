# Regular Expression Expectations

Scrut `(regex)` expectations support regular expressions to match output line(s).

This test file show-cases the use.

## Exact regex match of whole line

```scrut
$ echo foo
foo (regex)
```

## Regex match doesn't care about line ending

```scrut
$ echo -n foo
foo (regex)
```

## Regex match with wildcards

```scrut
$ echo foo
.* (regex)
```

## Regex match of multiple lines

```scrut
$ echo -e "foo\nfun\nfacts"
f.* (regex+)
```

## Regex with escaped characters

```scrut
$ echo "a [word]"
a \[word\] (regex)
```

## Regex with unnecessary escapes

```scrut
$ echo 'a "thing"'
a \"thing\" (regex)
```

## Regex with quantifiers

```scrut
$ echo aaaaa
a{5} (regex)
```

## Regex auto fix of curly braces

```scrut
$ echo '{"i am": "json"}'
{"i am": "json"} (regex)
```

Unescaped, matching curly braces are auto-escaped in pre-processing by Scrut.
