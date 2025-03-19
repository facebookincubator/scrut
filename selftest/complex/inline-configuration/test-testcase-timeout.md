# Validate per-testcase timeout configuration

Tests in this file validate that the `timeout` configuration modifies how long a test may run before the execution is aborted.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Run tests that timeout

```scrut
$ scrut_test "$TESTDIR"/test-testcase-timeout.mdtest 2>&1
// =============================================================================
// @ *test-testcase-timeout.mdtest:16 (glob)
// -----------------------------------------------------------------------------
// # Run test that times out
// -----------------------------------------------------------------------------
// $ echo Before2 && sleep 0.5 && echo After2
// =============================================================================

timeout in execution

## STDOUT
#> Before2
## STDERR


Result: 1 file(s) with 4 test(s): 1 succeeded, 1 failed and 2 skipped
[50]
```
