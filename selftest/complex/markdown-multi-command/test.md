# Proof multi-command in Markdown not supported

This test executes the failing test `multi-command-markdown-fail.md`, to prove
that indeed multiple commands are not supported in a single code block, because:

## Everything after the first command in a code block is considered output

```scrut
$ "${SCRUT_BIN}" test --match-markdown "*.mdtest" "$TESTDIR"/no-multiple-commands-in-code-block.mdtest
// =============================================================================
// @ *no-multiple-commands-in-code-block.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # Multiple commands in a single code block
// -----------------------------------------------------------------------------
// $ echo Foo
// =============================================================================

1  1  |   Foo
2     | - $ echo Bar
3     | - Bar


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```
