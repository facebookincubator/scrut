# Interpolation

## Export a variable

```scrut
$ export FOO=bar
```

## Variable resolved in shell expression

```scrut
$ echo "Hello $FOO"
Hello bar
```

## Variable resolved in output expectation with interpolation enabled

```scrut {interpolated: true}
$ echo "Hello $FOO"
Hello $FOO
```

## Variable not resolved in output expectation with interpolation disabled

```scrut
$ echo "Hello \$FOO"
Hello $FOO
```
