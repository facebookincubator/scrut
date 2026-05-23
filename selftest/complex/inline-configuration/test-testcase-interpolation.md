# Interpolation

## Export a variable

```mooncram
$ export FOO=bar
```

## Variable resolved in shell expression

```mooncram
$ echo "Hello $FOO"
Hello bar
```

## Variable resolved in output expectation with interpolation enabled

```mooncram {interpolated: true}
$ echo "Hello $FOO"
Hello $FOO
```

## Variable not resolved in output expectation with interpolation disabled

```mooncram
$ echo "Hello \$FOO"
Hello $FOO
```
