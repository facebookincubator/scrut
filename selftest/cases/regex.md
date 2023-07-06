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

## Regex match of multiple set of repeating lines
```scrut
$ echo -e "sea\nsand\nsea\nsand\nsea\nsand"
sea|sand (regex+)
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

## Table output

```scrut
$ cat <<EOF
> +------------+----------+----------+-------------+-------------------------+-----------+------------+
> |            | Saucisse | Beaufort | Raclette    | Omelette du fromage     | Roquefort | Emmental   |
> +------------+----------+----------+-------------+-------------------------+-----------+------------+
> | boo        | 59       | 61       | 61          | 13                      | 43        | 22         |
> +------------+----------+----------+-------------+-------------------------+-----------+------------+
> | bar        | 80       | 82       | 82          | 14                      | 49        | 11         |
> +------------+----------+----------+-------------+-------------------------+-----------+------------+
> | baz        | 91       | 91       | 91          | 9                       | 22        | 2          |
> +------------+----------+----------+-------------+-------------------------+-----------+------------+
> EOF
\+[-+]+\+ (regex)
.*Saucisse.*Raclette.* (regex)
\+[-+]+\+ (regex)
(\| b.\.*|)|(\+[-+]+\+) (regex+)
```
