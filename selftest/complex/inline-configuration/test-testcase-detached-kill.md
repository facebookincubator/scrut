# Validate per-testcase detached configuration

Test that validates that configured `detached_kill_signal` is send to process of `detached: true` test cases.

Works only on Linux and macOS

```mooncram
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80
```

Setup

```mooncram
$ alias moon_cram_test='$MOON_CRAM_BIN test --match-markdown="*.mdtest"'
```

## Run the test that spawns a detached process

```mooncram
$ export DELEGATED_TMPDIR="$TMPDIR" && \
>   moon_cram_test "$TESTDIR"/test-testcase-detached-kill.mdtest
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

## Detached process received SIGINT

```mooncram {wait: {path: "signal", timeout: 5s}}
$ cat "$TMPDIR"/signal
INT received
```

## Detached process terminated itself

```mooncram {wait: {path: "exit", timeout: 5s}}
$ cat "$TMPDIR"/exit
OK
```

## Detached process PID gone

```mooncram {wait: {path: "pid", timeout: 5s}}
$ kill -0 $(cat "$TMPDIR"/pid)
[1]
```
