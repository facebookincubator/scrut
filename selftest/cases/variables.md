# Behavior of variables

This test illustrates the behavior of variables

## Variables can be set

```scrut
$ SOME_VAR_1=some-value-1
```

## Variables can be exported

```scrut
$ export SOME_VAR_2=some-value-2
```

## Set variables can be accessed

```scrut
$ echo "value: $SOME_VAR_1"
value: some-value-1
```

## Exported variables can be accessed

```scrut
$ echo "value: $SOME_VAR_2"
value: some-value-2
```

## Exported variables are NOT in the environment

Caveat: For bash < 4 currently all variables become exported variables, hence no test.

## Exported variables are in the environment

```scrut
$ env | grep SOME_VAR_2
SOME_VAR_2=some-value-2
```
