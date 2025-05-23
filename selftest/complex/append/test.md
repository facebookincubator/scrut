# Test appending of test files

Scrut supports appending of tests in front of a list of other files.

## Run test without append

```scrut
$ "${SCRUT_BIN}" test --work-directory "$(pwd)" --match-markdown "*.mdtest" "$TESTDIR/actual.mdtest"
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

There should be a file still in place afterwards

```scrut
$ test -f some-file && echo "File exists"
File exists
```

## Run test with append

```scrut
$ "${SCRUT_BIN}" test --work-directory "$(pwd)" --match-markdown "*.mdtest" --append-test-file-paths "$TESTDIR/append.mdtest" "$TESTDIR/actual.mdtest"
Result: 0 document(s) with 0 testcase(s): 0 succeeded, 0 failed and 0 skipped
```

There should be no file around afterwards

```scrut
$ test -f some-file || echo "File was removed"
```
