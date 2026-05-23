# Validate per-testcase wait configuration

Tests in this file validate that `wait` will delay test execution until given path exists.

## Init timer

```mooncram
$ export TIME0=$(date +%s)
```

## Create a file in the background

```mooncram {detached: true}
$ sleep 1 && touch "$TMPDIR"/a-file
```

## Wait until file exists

```mooncram {wait: {timeout: 5s, path: a-file}}
$ WAITED=$(($(date +%s) - $TIME0))
> ( [ $WAITED -ge 1 ] && [ $WAITED -le 2 ] && echo "Waited about one seconds" ) || echo "Wait time unexpected: 1 <= $WAITED <= 2 is not true"
Waited about one seconds
```
