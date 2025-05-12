# Validate per-testcase detached configuration

Test that validates without an explicit `detached_kill_signal` tests that are `detached: true` are NOT killed automatically.

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
>   scrut_test "$TESTDIR"/test-testcase-detached-no-kill.mdtest
Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

## Detached process is still running

```scrut {wait: {path: "pid", timeout: 5s}}
$ kill -0 $(cat "$TMPDIR"/pid)
```

## Send signal to process

```scrut
$ kill -SIGINT $(cat "$TMPDIR"/pid)
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

```scrut
$ kill -0 $(cat "$TMPDIR"/pid)
[1]
```
