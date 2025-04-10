# Test conversion of test files in between formats

## Create a markdown test

```scrut
$ "${SCRUT_BIN}" create --format cram "echo Hello World" > test.t
```

Created Cram test file looks like this:

```scrut
$ cat test.t
Command executes successfully
  $ echo Hello World
  Hello World
```

## Convert from Markdown to Cram

```scrut
$ "${SCRUT_BIN}" update --convert markdown test.t
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

Converted Markdown test file looks like this:

````scrut
$ cat test.md
# Command executes successfully

```scrut {output_stream: combined, keep_crlf: true}
$ echo Hello World
Hello World
```
````
