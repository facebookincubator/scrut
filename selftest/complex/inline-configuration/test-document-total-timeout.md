# Validate the per-document total_timeout configuration

Tests in this file validate that the `total_timeout` configuration and the respective `--timeout-seconds` command line parameter act as expected.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Success without constraints

```scrut
$ scrut_test "$TESTDIR"/test-document-total-no-timeout.mdtest
Result: 1 file(s) with 3 test(s): 3 succeeded, 0 failed and 0 skipped
```

## Timeout with inline config

```scrut
$ scrut_test "$TESTDIR"/test-document-total-timeout.mdtest 2>&1
* failing in "*test-document-total-timeout.mdtest": timeout in executing (glob)
* (glob*)
[1]
```

## Timeout with command line parameter

```scrut
$ scrut_test "$TESTDIR"/test-document-total-no-timeout.mdtest --timeout-seconds 1 2>&1
* failing in "*test-document-total-no-timeout.mdtest": timeout in executing (glob)
* (glob*)
[1]
```
