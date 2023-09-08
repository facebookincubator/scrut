# Test user provided shell is used

This test proves that the `--work-directory` parameter runs in a single directory.

## Run test with temporary work directory

```scrut
$ "$SCRUT_BIN" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
// =============================================================================
// @ *test.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # Execution from within custom shell
// -----------------------------------------------------------------------------
// $ ls
// =============================================================================

   1  | - temp.* (glob)
   2  | - test-me.fixture


Summary: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Run test with custom work directory

```scrut
$ "$SCRUT_BIN" test --work-directory "$TESTDIR/fixtures" --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

## Run test with custom work directory (global)

```scrut
$ "$SCRUT_BIN" --work-directory "$TESTDIR/fixtures" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

Ensure idempotent execution by cleaning up fixtures directory

```scrut
$ rm -rf "$TESTDIR/fixtures/temp.*"
```
