# Regular Expression Expectations

Moon Cram `(regex)` expectations support regular expressions to match output line(s).

This test file show-cases the use.

## Exact regex match of whole line

```mooncram
$ echo foo
foo (regex)
```

## Regex match doesn't care about line ending

```mooncram
$ echo -n foo
foo (regex)
```

## Regex match with wildcards

```mooncram
$ echo foo
.* (regex)
```

## Regex match of multiple lines

```mooncram
$ echo -e "foo\nfun\nfacts"
f.* (regex+)
```

## Regex match of multiple set of repeating lines
```mooncram
$ echo -e "sea\nsand\nsea\nsand\nsea\nsand"
sea|sand (regex+)
```

## Regex with escaped characters

```mooncram
$ echo "a [word]"
a \[word\] (regex)
```

## Regex with unnecessary escapes

```mooncram
$ echo 'a "thing"'
a \"thing\" (regex)
```

## Regex with quantifiers

```mooncram
$ echo aaaaa
a{5} (regex)
```

## Regex auto fix of curly braces

```mooncram
$ echo '{"i am": "json"}'
{"i am": "json"} (regex)
```

Unescaped, matching curly braces are auto-escaped in pre-processing by Moon Cram.

## Table output

```mooncram
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
