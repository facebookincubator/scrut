# Test cram compat mode

Assure that the `--cram-compat` flag enables Cram backwards compatibility features.

## Cram compat mode is disabled for Markdown per default

```scrut
$ $SCRUT_BIN test --match-markdown "*.mdtest" "$TESTDIR/test-without-cr.mdtest" 2>&1
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

Scrut treats `\r` (LF) and `\r\n` (CRLF) as line breaks.

## Cram compat mode is disabled for Markdown files per default

```scrut
$ $SCRUT_BIN test --match-markdown "*.mdtest" --cram-compat "$TESTDIR/test-with-cr.mdtest" 2>&1
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

Scrut requires explicit handling of `\r` (CR) line breaks.

## Cram compat mode is enabled for Cram files per default

```scrut
$ $SCRUT_BIN test --match-cram "*.cram" "$TESTDIR/test.cram" 2>&1
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

Cram files auto-enable cram compat mode
