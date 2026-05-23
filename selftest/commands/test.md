# Create Command

## Bootstrap

```mooncram
$ . "${TESTDIR}/setup.sh"
OK
```

## Output of test -h in markdown format

```mooncram
$ "${MOON_CRAM_BIN}" test -h
Run tests from files or directories

Usage: moon-cram(?:\.exe)? test \[OPTIONS\] \[TEST_FILE_PATHS\]\.\.\. (regex)

Arguments:
  [TEST_FILE_PATHS]...  Path to test files or directories

Options:
* (glob+)
```
