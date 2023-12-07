# Test conversion of test files in between formats

## Create a markdown test

```scrut
$ "${SCRUT_BIN}" create --format markdown "echo Hello World" > test.md
```

Check the created Markdown test file

````scrut
$ cat test.md
# Command executes successfully

```scrut
$ echo Hello World
Hello World
```
````

## Convert from Markdown to Cram

```scrut
$ "${SCRUT_BIN}" update --convert cram test.md
Summary: 1 file(s) of which 1 updated, 0 skipped and 0 unchanged
```

A new Cram file should have been created and the tests should look fine:

```scrut
$ echo Hello World
Hello World
```
