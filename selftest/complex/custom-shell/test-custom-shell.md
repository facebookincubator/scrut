# Test user provided shell is used

This test proves that the `--shell` parameter provided shell is being used to execute the shell expressions of the tests in.

## Run test with standard shell

```scrut
$ "$SCRUT_BIN" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
// =============================================================================
// @ *test.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # Execution from within custom shell
// -----------------------------------------------------------------------------
// $ echo "This is from a custom shell: ${FROM_A_CUSTOM_SHELL:-no}"
// =============================================================================

   1  | - This is from a custom shell: yes
1     | + This is from a custom shell: no


Summary: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Run test with custom shell

```scrut
$ "$SCRUT_BIN" test --shell "$TESTDIR/shell.sh" --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

## Run test with custom shell (global)

```scrut
$ "$SCRUT_BIN" --shell "$TESTDIR/shell.sh" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```
