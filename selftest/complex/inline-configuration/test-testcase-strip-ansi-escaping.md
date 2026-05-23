# Validate per-testcase strip_ansi_escaping configuration

Tests in this file validate that the `strip_ansi_escaping` configuration makes Moon Cram ignore ANSI escape sequences in the test output validation.

Setup

```mooncram
$ alias moon_cram_test='$MOON_CRAM_BIN test --match-markdown="*.mdtest"'
```

## Run tests that timeout

```mooncram
$ moon_cram_test "$TESTDIR"/test-testcase-strip-ansi-escaping.mdtest 2>&1
Result: 1 document(s) with 2 testcase(s): 2 succeeded, 0 failed and 0 skipped
```
