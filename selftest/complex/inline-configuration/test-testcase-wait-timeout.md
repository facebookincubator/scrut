# Validate per-testcase wait configuration

Tests in this file validate that `wait` will delay test execution for given time.

## Init timer

```scrut
$ export TIME0=$(date +%s)
```

## Wait for a couple of seconds

```scrut {wait: 3s}
$ export WAITED=$(($(date +%s) - $TIME0))
> ( [ $WAITED -ge 3 ] && [ $WAITED -le 4 ] && echo "Waited about three seconds" ) || echo "Wait time unexpected: 3 <= $WAITED <= 4 is not true"
Waited about three seconds
```
