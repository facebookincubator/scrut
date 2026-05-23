# Create Command

## Bootstrap

```mooncram
$ . "${TESTDIR}/setup.sh"
OK
```

## Output of create -h in markdown format

```mooncram
$ "${MOON_CRAM_BIN}" create -h
Create tests from provided shell expression

Usage: moon-cram(?:\.exe)? create \[OPTIONS\] <SHELL_EXPRESSION>\.\.\. (regex)

Arguments:
  <SHELL_EXPRESSION>...  Shell expression THAT WILL BE EXECUTED to automatically
                         create a test from. Use "-" to read from STDIN

Options:
* (glob+)
```
