# Create Command

## Bootstrap

```scrut
$ . "${TESTDIR}/setup.sh"
OK
```

## Output of update -h in markdown format

```scrut
$ "${SCRUT_BIN}" update -h
Re-run all testcases in given file(s) and update the output expectations

Usage: scrut(?:\.exe)? update \[OPTIONS\] \<PATHS\>\.\.\. (regex)

Arguments:
  <PATHS>...  Path to test files or directories

Options:
* (glob+)
```
