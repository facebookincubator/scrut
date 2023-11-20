# Validate per-testcase detached configuration

Tests in this file validate that `detached` makes Scrut start executions, but not validate the result nor wait for it to finish.

## Run something detached

```scrut {detached: true}
$ echo "Starting HERE" && sleep 1 && echo FOO > "$TMPDIR"/output
```

This will not block and all output is ignored

## Without waiting, the detached thing is still not done

```scrut
$ test ! -f "$TMPDIR"/output && echo Detached is still running || ( echo Detached has already finished && ls -lha "$TMPDIR" )
Detached is still running
```

This ensures that at most one second passed since the start time was measured, which implies that neither

## With waiting, the detached thing has finished

```scrut
$ sleep 2 && test -f "$TMPDIR"/output && echo Detached has finished || ( echo Detached has not yet finished && ls -lha "$TMPDIR" )
Detached has finished
```
