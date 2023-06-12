# Create Command

## Bootstrap

```scrut
$ . "${TESTDIR}/setup.sh"
OK
```

## Output of create -h in markdown format

```scrut
$ "${SCRUT_BIN}" create -h
Create tests from provided shell expression

Usage: scrut(?:\.exe)? create \[OPTIONS\] <SHELL_EXPRESSION>\.\.\. (regex)

Arguments:
  <SHELL_EXPRESSION>...  Shell expression THAT WILL BE EXECUTED to automatically
                         create a test from. Use "-" to read from STDIN

Options:
* (glob+)
```
