# A test that use user-provided packages

This test validates that provided RPMs and FPBKGs are indeed available.

Load setup

```scrut
$ source "$TESTDIR/setup.sh"
```

## Is RPM available?

```scrut
$ cli --help
* (glob+)
Usage: ohno <COMMAND>
* (glob+)
```

## Is New-Style FBPKG available?

```scrut
$ test -d /packages/biggrep.client
```


## Is Old-Style FBPKG available?

```scrut
$ test -d /packages/hwcontrol.cli
```
