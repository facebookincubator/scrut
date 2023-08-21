# Bash vs output streams

This test documents the behavior of Scrut executions in regards to backgrounding / detaching and open streams (STDOUT, STDERR, STDIN):
The `bash` process that executes a Scrut test expression will not close as long as any standard streams (`/dev/stdin`, `/dev/stdout` or `/dev/stderr`) are open.

## Waiting for standard streams

```scrut
$ export START_TIME=$(date +%s)
```

Sleep without manipulating standard streams, which will automatically use `/dev/std{err,out}`.

```scrut
$ sleep 5 &
```

Scrut waited for the whole 5 seconds

```scrut
$ CURRENT_TIME=$(date +%s)
> DURATION=$((CURRENT_TIME - START_TIME))
> ( test $DURATION -ge 5 && test $DURATION -le 6 ) || echo "Execution should have waited for 5-6 seconds, but waited $DURATION"
```

## No wait when standard streams are unused

```scrut
$ export START_TIME=$(date +%s)
```

Sleep with redirecting standard streams, so that `/dev/std{err,out}` are not used.

```scrut
$ sleep 5 1>/dev/null 2>/dev/null &
```

Bash process was not blocked by waiting for STD{ERR,OUT}, hence Scrut did not wait for the whole 5 seconds.

```scrut
$ CURRENT_TIME=$(date +%s)
> DURATION=$((CURRENT_TIME - START_TIME))
> test $DURATION -le 2 || echo "2 Should not have waited for sleep to finish, but waited $DURATION seconds"
```
