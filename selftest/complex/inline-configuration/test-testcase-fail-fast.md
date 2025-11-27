# Validate per-testcase fail_fast configuration

Tests in this file validate that the `fail_fast` option stops execution of the entire test document when a test case fails.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## fail_fast stops on first failure

```scrut
$ scrut_test "$TESTDIR"/test-testcase-fail-fast.mdtest 2>&1
// =============================================================================
// @ *test-testcase-fail-fast.mdtest:* (glob)
// -----------------------------------------------------------------------------
// # Second test fails with fail_fast
// -----------------------------------------------------------------------------
// $ echo "Test 2"
// =============================================================================

1     | - Wrong output
   1  | + Test 2


Result: 1 document(s) with 3 testcase(s): 1 succeeded, 1 failed and 1 skipped
[50]
```

## without fail_fast all tests run

```scrut
$ scrut_test "$TESTDIR"/test-testcase-no-fail-fast.mdtest 2>&1
// =============================================================================
// @ *test-testcase-no-fail-fast.mdtest:* (glob)
// -----------------------------------------------------------------------------
// # Second test fails without fail_fast
// -----------------------------------------------------------------------------
// $ echo "Test 2"
// =============================================================================

1     | - Wrong output
   1  | + Test 2


Result: 1 document(s) with 3 testcase(s): 2 succeeded, 1 failed and 0 skipped
[50]
```
