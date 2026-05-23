# Test the `update` command

Validate the generated output of the `update` command.

```mooncram
$ alias moon_cram_run='"$MOON_CRAM_BIN" update --log-level info --verbose --no-color --replace --assume-yes --match-markdown "*.mdtest"'
```

## Succeeding tests

```mooncram
$ cp "$TESTDIR/fixtures/ok1.mdtest" "$TMPDIR/" && \
>  moon_cram_run "$TMPDIR/ok1.mdtest" 2>&1
* INFO moon_cram::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO moon_cram::utils::ui: 👀 *ok1.mdtest (glob)
* INFO moon_cram::utils::ui: 👍 *ok1.mdtest: keep as-is, no changes in document content (glob)
Result: 1 document(s) of which 0 updated, 0 skipped and 1 unchanged
```

## Failing tests

```mooncram
$ cp "$TESTDIR/fixtures/err1.mdtest" "$TMPDIR/" && \
>  moon_cram_run "$TMPDIR/err1.mdtest" 2>&1
* INFO moon_cram::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO moon_cram::utils::ui: 👀 *err1.mdtest (glob)
// =============================================================================
// @ *err1.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # A failing test 1
// -----------------------------------------------------------------------------
// $ echo OK
// =============================================================================

1     | - Fail
   1  | + OK


* INFO moon_cram::utils::ui: ✍️ *err1.mdtest: overwritten document with updated contents (glob)
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

Failing test is updated

````mooncram
$ cat "$TMPDIR/err1.mdtest"
# A failing test 1

```mooncram
$ echo OK
OK
```
````
