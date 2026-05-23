# Test that Moon Cram proposes noeol if a line does not match for that reason

This test validates that if the user writes a Moon Cram test where they use an equal-rule (i.e. with ending new-line
character) but the output does not end in a new-line, then Moon Cram prints the missing `(no-eol)` suffix.

## No-EOL is proposed in the pretty print of the failed test

```mooncram
$ "$MOON_CRAM_BIN" test --renderer pretty --match-markdown "*.mdtest" "$TESTDIR/invalid.mdtest"
// =============================================================================
// @ .*[/\\]invalid\.mdtest:4 (regex)
// -----------------------------------------------------------------------------
// # A test where output does NOT end in new-line
// -----------------------------------------------------------------------------
// $ echo -n There is no new line
// =============================================================================

1     | - There is no new line
   1  | + There is no new line (no-eol) (equal)


// =============================================================================
// @ .*[/\\]invalid\.mdtest:11 (regex)
// -----------------------------------------------------------------------------
// # A test where output DOES end in new-line
// -----------------------------------------------------------------------------
// $ echo There is a new line
// =============================================================================

1     | - There is a new line (no-eol) (equal)
   1  | + There is a new line


Result: 1 document(s) with 2 testcase(s): 0 succeeded, 2 failed and 0 skipped
[50]
```
