# Validate the per-document total_timeout configuration

Tests in this file validate that the `total_timeout` configuration and the respective `--timeout-seconds` command line parameter act as expected.

```mooncram
$ alias moon_cram_test='$MOON_CRAM_BIN test --match-markdown="*.mdtest"'
```

## Success without constraints

```mooncram
$ moon_cram_test "$TESTDIR"/test-document-total-no-timeout.mdtest
Result: 1 document(s) with 3 testcase(s): 3 succeeded, 0 failed and 0 skipped
```

## Timeout with inline config

```mooncram
$ moon_cram_test "$TESTDIR"/test-document-total-timeout.mdtest 2>&1
// =============================================================================
// @ *test-document-total-timeout.mdtest:20 (glob)
// -----------------------------------------------------------------------------
// # Run the second test
// -----------------------------------------------------------------------------
// $ echo TestB && sleep 0.5 && echo Test2
// =============================================================================

timeout in execution

## STDOUT
#> TestB
## STDERR


Result: 1 document(s) with 3 testcase(s): 1 succeeded, 1 failed and 1 skipped
[50]
```

## Timeout with command line parameter

```mooncram
$ moon_cram_test "$TESTDIR"/test-document-total-no-timeout.mdtest --timeout-seconds 1 2>&1
// =============================================================================
// @ *test-document-total-no-timeout.mdtest:16 (glob)
// -----------------------------------------------------------------------------
// # Run the second test
// -----------------------------------------------------------------------------
// $ echo TestB && sleep 0.5 && echo Test2
// =============================================================================

timeout in execution

## STDOUT
#> TestB
## STDERR


Result: 1 document(s) with 3 testcase(s): 1 succeeded, 1 failed and 1 skipped
[50]
```
