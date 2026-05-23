# Test user provided shell is used

This test proves that the `--shell` parameter provided shell is being used to execute the shell expressions of the tests in.

Does not work in windows

```mooncram
$ ( [[ "$(uname -s)" == "Darwin" ]] || [[ "$(uname -s)" == "Linux" ]] ) || exit 80
```

## Run test with standard shell

```mooncram
$ "$MOON_CRAM_BIN" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
// =============================================================================
// @ *test.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # Execution from within custom shell
// -----------------------------------------------------------------------------
// $ echo "This is from a custom shell: ${FROM_A_CUSTOM_SHELL:-no}"
// =============================================================================

1     | - This is from a custom shell: yes
   1  | + This is from a custom shell: no


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Run test with custom shell

```mooncram
$ "$MOON_CRAM_BIN" test --shell "$TESTDIR/shell.sh" --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

## Run test with custom shell (global)

```mooncram
$ "$MOON_CRAM_BIN" --shell "$TESTDIR/shell.sh" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```
