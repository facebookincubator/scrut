# Test that Scrut proposes noeol if a line does not match for that reason

This test validates that if the user writes a Scrut test where they use an equal-rule (i.e. with ending new-line
character) but the output does not end in a new-line, then Scrut prints the missing `(no-eol)` suffix.


```scrut
$ cp "$TESTDIR/invalid.mdtest" "$(pwd)/invalid.mdtest"
```

## No-EOL is proposed in the pretty print of the failed test

```scrut
$ "$SCRUT_BIN" update --match-markdown "*.mdtest" "$(pwd)/invalid.mdtest"
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

## Generated file matches valid result

```scrut
$ diff "$(pwd)/invalid.mdtest.new" "$TESTDIR/valid.mdtest"
```
