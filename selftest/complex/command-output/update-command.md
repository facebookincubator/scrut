# Test the `update` command

Validate the generated output of the `update` command.

```scrut
$ alias scrut_run='"$SCRUT_BIN" update --log-level info --verbose --no-color --replace --assume-yes --match-markdown "*.mdtest"'
```

## Succeeding tests

```scrut
$ cp "$TESTDIR/fixtures/ok1.mdtest" "$TMPDIR/" && \
>  scrut_run "$TMPDIR/ok1.mdtest" 2>&1
* INFO scrut::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO scrut::utils::ui: 👀 *ok1.mdtest (glob)
* INFO scrut::utils::ui: 👍 *ok1.mdtest: keep as-is, no changes in document content (glob)
Result: 1 document(s) of which 0 updated, 0 skipped and 1 unchanged
```

## Failing tests

```scrut
$ cp "$TESTDIR/fixtures/err1.mdtest" "$TMPDIR/" && \
>  scrut_run "$TMPDIR/err1.mdtest" 2>&1
* INFO scrut::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO scrut::utils::ui: 👀 *err1.mdtest (glob)
// =============================================================================
// @ *err1.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # A failing test 1
// -----------------------------------------------------------------------------
// $ echo OK
// =============================================================================

1     | - Fail
   1  | + OK


* INFO scrut::utils::ui: ✍️ *err1.mdtest: overwritten document with updated contents (glob)
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

Failing test is updated

````scrut
$ cat "$TMPDIR/err1.mdtest"
# A failing test 1

```scrut
$ echo OK
OK
```
````
