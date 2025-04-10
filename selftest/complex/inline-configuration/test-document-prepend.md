# Validate the per-document prepend configuration

Tests in this file validate that the `prepend` configuration and the respective `--prepend-test-file-paths` command line parameter act as expected.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Inline prepending

```scrut
$ scrut_test "$TESTDIR"/test-document-prepend.mdtest
// =============================================================================
// @ *test-document-prepend.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # This fails in prepended
// -----------------------------------------------------------------------------
// $ echo FooInPrepended1
// =============================================================================

1     | - BarInPrepended1
   1  | + FooInPrepended1


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Inline and command line prepending

```scrut
$ scrut_test "$TESTDIR"/test-document-prepend.mdtest --prepend-test-file-paths "$TESTDIR"/test-document-prepended-2.mdtest
// =============================================================================
// @ *test-document-prepend.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # This fails in prepended
// -----------------------------------------------------------------------------
// $ echo FooInPrepended2
// =============================================================================

1     | - BarInPrepended2
   1  | + FooInPrepended2


// =============================================================================
// @ *test-document-prepend.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # This fails in prepended
// -----------------------------------------------------------------------------
// $ echo FooInPrepended1
// =============================================================================

1     | - BarInPrepended1
   1  | + FooInPrepended1


Result: 1 document(s) with 2 testcase(s): 0 succeeded, 2 failed and 0 skipped
[50]
```

Command line prepend is added before the inline prepend.
