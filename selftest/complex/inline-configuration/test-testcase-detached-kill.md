# Validate per-testcase detached configuration

Test that validates that configured `detached_kill_signal` is send to process of `detached: true` test cases.

Works only on Linux and macOS

```scrut
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80
```

Setup

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Run the test that spawns a detached process

```scrut
$ export DELEGATED_TMPDIR="$TMPDIR" && \
>   scrut_test "$TESTDIR"/test-testcase-detached-kill.mdtest
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

## Detached process received SIGINT

```scrut {wait: {path: "signal", timeout: 5s}}
$ cat "$TMPDIR"/signal
INT received
```

## Detached process terminated itself

```scrut {wait: {path: "exit", timeout: 5s}}
$ cat "$TMPDIR"/exit
OK
```

## Detached process PID gone

```scrut {wait: {path: "pid", timeout: 5s}}
$ kill -0 $(cat "$TMPDIR"/pid)
[1]
```
