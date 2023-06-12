# Create Command

## Bootstrap

```scrut
$ . "${TESTDIR}/setup.sh"
OK
```

## Output of test -h in markdown format

```scrut
$ "${SCRUT_BIN}" test -h
Run tests from files or directories

Usage: scrut(?:\.exe)? test \[OPTIONS\] \[TEST_FILE_PATHS\]\.\.\. (regex)

Arguments:
  [TEST_FILE_PATHS]...  Path to test files or directories

Options:
* (glob+)
```
