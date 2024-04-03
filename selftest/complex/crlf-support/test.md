# Test user provided shell is used

This test proves that the `--keep-output-crlf` flag combines STDOUT and STDERR into a single output stream that can be tested as one.

## Run with normal CRLF handling

```scrut
$ "$SCRUT_BIN" test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
// =============================================================================
// @ *test.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # Test expecting CRLF support
// -----------------------------------------------------------------------------
// $ echo -en "With\r\nNewlines\r\n"
// =============================================================================

   1  | - With\r (escaped) (equal)
   2  | - Newlines\r (escaped) (equal)
1     | + With
2     | + Newlines


Result: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Run with explicit CRLF handling

```scrut
$ "$SCRUT_BIN" test --keep-output-crlf --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Result: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

## Run with explicit CRLF handling (global)

```scrut
$ "$SCRUT_BIN" --keep-output-crlf test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Result: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```


## Run with cram compat enabling CRLF handling

```scrut
$ "$SCRUT_BIN" --cram-compat test --match-markdown "*.mdtest" "$TESTDIR/test.mdtest"
Result: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```
