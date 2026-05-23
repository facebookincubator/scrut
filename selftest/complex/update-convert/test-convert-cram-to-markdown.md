# Test conversion of test files in between formats

## Create a markdown test

```mooncram
$ "${MOON_CRAM_BIN}" create --format cram "echo Hello World" > test.t
```

Created Cram test file looks like this:

```mooncram
$ cat test.t
Command executes successfully
  $ echo Hello World
  Hello World
```

## Convert from Markdown to Cram

```mooncram
$ "${MOON_CRAM_BIN}" update --convert markdown test.t
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

Converted Markdown test file looks like this:

````mooncram
$ cat test.md
# Command executes successfully

```mooncram {output_stream: combined, keep_crlf: true}
$ echo Hello World
Hello World
```
````
