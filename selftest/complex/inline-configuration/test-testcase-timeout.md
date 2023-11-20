# Validate per-testcase timeout configuration

Tests in this file validate that the `timeout` configuration modifies how long a test may run before the execution is aborted.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Run tests that timeout

```scrut
$ scrut_test "$TESTDIR"/test-testcase-timeout.mdtest 2>&1
*failing in "*test-testcase-timeout.mdtest": timeout in executing shell expression of test 2 (glob)
* (glob*)
[1]
```
