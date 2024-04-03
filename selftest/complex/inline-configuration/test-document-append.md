# Validate the per-document append configuration

Tests in this file validate that the `append` configuration and the respective `--append-test-file-paths` command line parameter act as expected.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Inline appending

```scrut
$ scrut_test "$TESTDIR"/test-document-append.mdtest
// =============================================================================
// @ *test-document-append.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # This fails in appended
// -----------------------------------------------------------------------------
// $ echo FooInAppended1
// =============================================================================

   1  | - BarInAppended1
1     | + FooInAppended1


Result: 1 file(s) with 1 test(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Inline and command line appending

```scrut
$ scrut_test "$TESTDIR"/test-document-append.mdtest --append-test-file-paths "$TESTDIR"/test-document-appended-2.mdtest
// =============================================================================
// @ *test-document-append.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # This fails in appended
// -----------------------------------------------------------------------------
// $ echo FooInAppended1
// =============================================================================

   1  | - BarInAppended1
1     | + FooInAppended1


// =============================================================================
// @ *test-document-append.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # This fails in appended
// -----------------------------------------------------------------------------
// $ echo FooInAppended2
// =============================================================================

   1  | - BarInAppended2
1     | + FooInAppended2


Result: 1 file(s) with 2 test(s): 0 succeeded, 2 failed and 0 skipped
[50]
```

Command line append is added after the inline append.
