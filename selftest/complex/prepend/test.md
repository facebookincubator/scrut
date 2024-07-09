# Test prepending of test files

Scrut supports prepending of tests in front of a list of other files.

## Run tests without prepend should make it fail

```scrut
$ "${SCRUT_BIN}" test --match-markdown "*.mdtest" "$TESTDIR/actual.mdtest"
// =============================================================================
// @ *actual.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # Run test with the assumption that prepended.mdtest is injected
// -----------------------------------------------------------------------------
// $ echo "Good things are ${SOME_VARIABLE:-undefined}"
// =============================================================================

1     | - Good things are prepended, not shaken
   1  | + Good things are undefined


Result: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Run test with prepend should make it succeed

```scrut
$ "${SCRUT_BIN}" test --match-markdown "*.mdtest" --prepend-test-file-paths "$TESTDIR/prepend.mdtest" "$TESTDIR/actual.mdtest"
Result: 0 file(s) with 0 test(s): 0 succeeded, 0 failed and 0 skipped
```
