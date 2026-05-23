# Validate the per-document shell configuration

Tests in this file validate that the `shell` configuration and the respective `--shell` command line parameter act as expected.

```mooncram
$ alias moon_cram_test='$MOON_CRAM_BIN test --match-markdown="*.mdtest"'
```

## Inline shell

```mooncram
$ moon_cram_test "$TESTDIR"/test-document-shell.mdtest 2>&1
* guessing path to shell `something-really-invalid-inline` (glob)

Caused by:
    cannot find binary path
* (glob*)
[1]
```

## Override inline language markers by parameter

```mooncram
$ moon_cram_test "$TESTDIR"/test-document-shell.mdtest --shell that-does-not-exist-either 2>&1
* guessing path to shell `that-does-not-exist-either` (glob)

Caused by:
    cannot find binary path
* (glob*)
[1]
```
