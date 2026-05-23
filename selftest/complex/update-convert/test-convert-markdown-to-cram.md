# Test conversion of test files in between formats

## Create a markdown test

```mooncram
$ "${MOON_CRAM_BIN}" create --format markdown "echo Hello World" > test.md
```

Check the created Markdown test file

````mooncram
$ cat test.md
# Command executes successfully

```mooncram
$ echo Hello World
Hello World
```
````

## Convert from Markdown to Cram

```mooncram
$ "${MOON_CRAM_BIN}" update --convert cram test.md
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

A new Cram file should have been created and the tests should look fine:

```mooncram
$ echo Hello World
Hello World
```
