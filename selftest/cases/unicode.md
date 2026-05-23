# Unicode and ASCII support

Ensure that Moon Cram defaults to Unicode use, but can still deal with ASCII.

## Unicode in output is passed on

````mooncram
$ $MOON_CRAM_BIN create "echo -e \"foo 😁 bar\""
# Command executes successfully

```mooncram
$ echo -e "foo 😁 bar"
foo 😁 bar
```
````

## Non-printable characters are still escaped

````mooncram
$ $MOON_CRAM_BIN create "echo -e \"foo \\x1b[1m😁 bar\\x1b[0m baz\""
# Command executes successfully

```mooncram
$ echo -e "foo \x1b[1m😁 bar\x1b[0m baz"
foo \x1b[1m😁 bar\x1b[0m baz (escaped) (equal)
```
````

## Explicit ASCII escaping is honoured

````mooncram
$ $MOON_CRAM_BIN create --escaping=ascii "echo -e \"foo 😁 bar\""
# Command executes successfully

```mooncram
$ echo -e "foo 😁 bar"
foo \xf0\x9f\x98\x81 bar (escaped) (equal)
```
````

## Cram defaults to ASCII encoded

```mooncram
$ $MOON_CRAM_BIN create --format=cram "echo -e \"foo 😁 bar\""
Command executes successfully
  $ echo -e "foo 😁 bar"
  foo \xf0\x9f\x98\x81 bar (escaped) (equal)
```
