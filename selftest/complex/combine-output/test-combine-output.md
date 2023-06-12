# Test user provided shell is used

This test proves that the `--combine-output` flag combines STDOUT and STDERR into a single output stream that can be tested as one.

## Run test with normal output

```scrut
$ "$SCRUT_BIN" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
// =============================================================================
// @ *test.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # Execution from within custom shell
// -----------------------------------------------------------------------------
// $ echo "standard output" && ( 1>&2 echo "standard error" )
// =============================================================================

1  1  |   standard output
   2  | - standard error


Summary: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Run test with combined output

```scrut
$ "$SCRUT_BIN" test --combine-output --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

## Run test with combined output (global)

```scrut
$ "$SCRUT_BIN" --combine-output test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

## Run test with cram compat enabling combined output

```scrut
$ "$SCRUT_BIN" --cram-compat test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Summary: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```
