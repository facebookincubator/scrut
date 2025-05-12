# Test the `test` command

Validate the generated output of the `test` command.

```scrut
$ alias scrut_run='"$SCRUT_BIN" test --log-level info --verbose --no-color --match-markdown "*.mdtest"'
```

## Succeeding tests

```scrut
$ scrut_run "$TESTDIR/fixtures/"ok* 2>&1
* INFO scrut::utils::ui: 🔎 Found 2 test document(s) (glob)
* INFO scrut::utils::ui: 👀 *ok1.mdtest (glob)
* INFO scrut::utils::ui: ✅ *ok1.mdtest: passed 1 testcase (glob)
* INFO scrut::utils::ui: 👀 *ok2.mdtest (glob)
* INFO scrut::utils::ui: ✅ *ok2.mdtest: passed 1 testcase (glob)
* INFO scrut::commands::test: success=2 skipped=0 failed=0 detached=0 (glob)
Result: 2 document(s) with 2 testcase(s): 2 succeeded, 0 failed and 0 skipped
```

## Failing tests

```scrut
$ scrut_run "$TESTDIR/fixtures/"err* 2>&1
* INFO scrut::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO scrut::utils::ui: 👀 *err1.mdtest (glob)
* ERROR scrut::utils::ui: ❌ *err1.mdtest: failed 1 out of 1 testcase (glob)
* INFO scrut::commands::test: success=0 skipped=0 failed=1 detached=0 (glob)
// =============================================================================
// @ *err1.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # A failing test 1
// -----------------------------------------------------------------------------
// $ echo OK
// =============================================================================

1     | - Fail
   1  | + OK


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Timeout per document

```scrut
$ scrut_run "$TESTDIR/fixtures/timeout-per-document.mdtest" 2>&1
* INFO scrut::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO scrut::utils::ui: 👀 *timeout-per-document.mdtest (glob)
* WARN scrut::utils::ui: ⌛️ *timeout-per-document.mdtest: execution timed out after 1s at per-document timeout (glob)
* INFO scrut::commands::test: success=0 skipped=0 failed=1 detached=0 (glob)
// =============================================================================
// @ *timeout-per-document.mdtest:8 (glob)
// -----------------------------------------------------------------------------
// # A test that times out per-document
// -----------------------------------------------------------------------------
// $ echo Before; sleep 3; echo After
// =============================================================================

timeout in execution

## STDOUT
#> Before
## STDERR


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```

## Timeout per testcase

```scrut
$ scrut_run "$TESTDIR/fixtures/timeout-per-testcase.mdtest" 2>&1
* INFO scrut::utils::ui: 🔎 Found 1 test document(s) (glob)
* INFO scrut::utils::ui: 👀 *timeout-per-testcase.mdtest (glob)
* WARN scrut::utils::ui: ⌛️ *timeout-per-testcase.mdtest: execution timed out after 1s at per-testcase timeout in testcase #1 (glob)
* INFO scrut::commands::test: success=0 skipped=0 failed=1 detached=0 (glob)
// =============================================================================
// @ *timeout-per-testcase.mdtest:4 (glob)
// -----------------------------------------------------------------------------
// # A test that times out per-test
// -----------------------------------------------------------------------------
// $ echo Before; sleep 3; echo After
// =============================================================================

timeout in execution

## STDOUT
#> Before
## STDERR


Result: 1 document(s) with 1 testcase(s): 0 succeeded, 1 failed and 0 skipped
[50]
```
