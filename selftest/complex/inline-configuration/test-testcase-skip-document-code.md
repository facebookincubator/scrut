# Validate per-testcase skip_document_code configuration

Tests in this file validate that the `skip_document_code` controls which exit code signals to Scrut to skip (ignore) all tests in the document.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Default skip document code

```scrut
$ scrut_test "$TESTDIR"/test-testcase-skip-document-code-default.mdtest
Result: 1 file(s) with 3 test(s): 0 succeeded, 0 failed and 3 skipped
```

## Custom skip document code

```scrut
$ scrut_test "$TESTDIR"/test-testcase-skip-document-code-custom.mdtest
Result: 1 file(s) with 3 test(s): 0 succeeded, 0 failed and 3 skipped
```
