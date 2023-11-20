# Validate the per-document shell configuration

Tests in this file validate that the `shell` configuration and the respective `--shell` command line parameter act as expected.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Inline shell

```scrut
$ scrut_test "$TESTDIR"/test-document-shell.mdtest 2>&1
* guessing path to shell `something-really-invalid-inline` (glob)

Caused by:
    cannot find binary path
* (glob*)
[1]
```

## Override inline language markers by parameter

```scrut
$ scrut_test "$TESTDIR"/test-document-shell.mdtest --shell that-does-not-exist-either 2>&1
* guessing path to shell `that-does-not-exist-either` (glob)

Caused by:
    cannot find binary path
* (glob*)
[1]
```
