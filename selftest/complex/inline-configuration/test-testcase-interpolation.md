# Interpolation

## Export a variable

```scrut
$ export FOO=bar FOO_BAR=baz
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

## Braced variable with underscore resolved in output expectation

```scrut {interpolated: true}
$ echo "Hello $FOO_BAR"
Hello ${FOO_BAR}
```
