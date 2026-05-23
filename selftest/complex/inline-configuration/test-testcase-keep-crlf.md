# Validate per-testcase keep_crlf configuration

Tests in this file validate that the `keep_crlf` configuration modifies CRLF handling.

```mooncram
$ alias moon_cram_test='$MOON_CRAM_BIN test --match-markdown="*.mdtest"'
```

## Per default CRLF is treated as LF

```mooncram
$ echo -e "word\r"
word
```

## When enabled CR must be handled

```mooncram {keep_crlf: true}
$ echo -e "word\r"
word\r (escaped)
```

## Explicitly disabled

```mooncram
$ moon_cram_test "$TESTDIR"/test-testcase-keep-crlf-disabled.mdtest
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

## Explicitly enabled

```mooncram
$ moon_cram_test "$TESTDIR"/test-testcase-keep-crlf-disabled.mdtest --keep-output-crlf
// =============================================================================
// @ *test-testcase-keep-crlf-disabled.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # Run test
// -----------------------------------------------------------------------------
// $ echo -e "word\r"
// =============================================================================

1     | - word
   1  | + word\r (escaped) (equal)


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```
