# Create Command

## Bootstrap

```mooncram
$ . "${TESTDIR}/setup.sh"
OK
```

## Output of update -h in markdown format

```mooncram
$ "${MOON_CRAM_BIN}" update -h
Re-run all testcases in given file(s) and update the output expectations

Usage: moon-cram(?:\.exe)? update \[OPTIONS\] \<PATHS\>\.\.\. (regex)

Arguments:
  <PATHS>...  Path to test files or directories

Options:
* (glob+)
```
