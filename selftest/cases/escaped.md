# Escaped Expectations

Moon Cram `(escaped)` expectations support escaped representations of (otherwise) unprintable characters.

This test file show-cases the use.

## Escaped color code

```mooncram
$ echo -e "foo \033[1mbar\033[0m baz"
foo \x1b[1mbar\x1b[0m baz (escaped)
```

## Escaped color code shorthand

```mooncram
$ echo -e "foo \033[1mbar\033[0m baz"
foo \e[1mbar\e[0m baz (escaped)
```

## Escape unprintable character

```mooncram
$ echo -e 'foo\003 bar'
foo\x03 bar (escaped)
```

## Double escapes are fine


```mooncram
$ echo -e 'foo\\nbar'
foo\\nbar (escaped)
```

## Escaped Alarm Bell

```mooncram
$ echo -e 'foo\abar\abaz'
foo\abar\abaz (escaped)
```

## Escaped Overprint

```mooncram
$ echo -e 'foo\bbar\bbaz'
foo\bbar\bbaz (escaped)
```

## Escaped Form Feed

```mooncram
$ echo -e 'foo\fbar\fbaz'
foo\fbar\fbaz (escaped)
```

## Escaped Carriage Return (CR)

```mooncram
$ echo -e 'foo\rbar\rbaz'
foo\rbar\rbaz (escaped)
```

## Escaped Horizontal Tab

```mooncram
$ echo -e "foo\tbar\tbaz"
foo\tbar\tbaz (escaped)
```

## Escaped Vertical Tab

```mooncram
$ echo -e 'foo\vbar\vbaz'
foo\vbar\vbaz (escaped)
```

## Don't escape unicode

```mooncram
$ echo -e 'foo 😁\n🦀 bar\nbaz 🤡'
foo 😁
🦀 bar
baz 🤡
```

## Escaped mixed with unicode

```mooncram
$ echo -e 'foo \033[1m😁\t🦀 bar\033[0m\nbaz 🤡'
foo \x1b[1m😁\t🦀 bar\x1b[0m (escaped)
baz 🤡
```
