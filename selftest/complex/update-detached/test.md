# Update tests that use detached

Tests in this file validate that `update` commands on test files that contain `detached: true` tests leaves them unchanged.

```scrut
$ alias scrut_update='$SCRUT_BIN update --match-markdown="*.mdtest"'
```

## Create backup to compar against later

```scrut
$ cp "$TESTDIR"/test.mdtest ./test-copy.mdtest
```

## Run update

```scrut
$ scrut_update --replace --assume-yes "$TESTDIR"/test.mdtest
Result: 1 document(s) of which 0 updated, 0 skipped and 1 unchanged
```

## File ought to be unchanged

```scrut
$ diff "$TESTDIR"/test.mdtest ./test-copy.mdtest
```
