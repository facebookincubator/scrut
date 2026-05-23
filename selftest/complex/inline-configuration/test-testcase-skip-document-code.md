# Validate per-testcase skip_document_code configuration

Tests in this file validate that the `skip_document_code` controls which exit code signals to Moon Cram to skip (ignore) all tests in the document.

```mooncram
$ alias moon_cram_test='$MOON_CRAM_BIN test --match-markdown="*.mdtest"'
```

## Default skip document code

```mooncram
$ moon_cram_test "$TESTDIR"/test-testcase-skip-document-code-default.mdtest
Result: 1 document(s) with 3 testcase(s): 0 succeeded, 0 failed and 3 skipped
```

## Custom skip document code

```mooncram
$ moon_cram_test "$TESTDIR"/test-testcase-skip-document-code-custom.mdtest
Result: 1 document(s) with 3 testcase(s): 0 succeeded, 0 failed and 3 skipped
```
