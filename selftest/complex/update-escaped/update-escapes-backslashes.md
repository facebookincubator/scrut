# Test that updated escaped expectations match the output

This test validates that if an output line contains a non-printable character (here a tab) and a backslash, then
`scrut update` escapes the backslash as well, so the written `(escaped)` expectation matches the output it was
generated from.


```scrut
$ cp "$TESTDIR/backslash.mdtest" "$(pwd)/backslash.mdtest"
```

## Update writes an escaped expectation

```scrut
$ "$SCRUT_BIN" update --match-markdown "*.mdtest" "$(pwd)/backslash.mdtest"
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

## The backslash is escaped in the generated file

```scrut
$ grep -F '(escaped)' "$(pwd)/backslash.mdtest.new"
C:\\temp\tdone (escaped) (equal)
```

## The generated file passes

```scrut
$ cp "$(pwd)/backslash.mdtest.new" "$(pwd)/updated.mdtest" && \
>   "$SCRUT_BIN" test --match-markdown "*.mdtest" "$(pwd)/updated.mdtest"
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```
