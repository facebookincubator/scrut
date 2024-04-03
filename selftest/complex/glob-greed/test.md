# Glob Greed

Assure the glob rule is only as greed as it should be

## Succeed with valid concrete match sandwiched between greedy multiline

```scrut
$ "$SCRUT_BIN" test --match-markdown "*.mdtest" "$TESTDIR/greedy-good.mdtest"
Result: 1 file(s) with 1 test(s): 1 succeeded, 0 failed and 0 skipped
```

## Fail with invalid concrete match sandwiched between greedy multiline

```scrut
$ "$SCRUT_BIN" test --match-markdown "*.mdtest" "$TESTDIR/greedy-bad.mdtest"
// =============================================================================
// @ *greedy-bad.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # This test must fail
// -----------------------------------------------------------------------------
// $ echo -e "foo\nbar\nbaz"
// =============================================================================

+  1+ |   * (glob*)
   2  | - bla


Result: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```
