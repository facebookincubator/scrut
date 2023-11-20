# Validate per-testcase environment configuration

Tests in this file validate that the `environment` configuration sets environment variables as intended.

```scrut
$ alias scrut_test='$SCRUT_BIN test --match-markdown="*.mdtest"'
```

## Environment variable is not set initially

```scrut
$ echo "Var is '${SOME_VAR}'"
Var is ''
```

## Environment variable is set

```scrut {environment: {"SOME_VAR": "some value"}}
$ echo "Var is '${SOME_VAR}'"
Var is 'some value'
```

## Environment variable will not be unset

```scrut
$ echo "Var is '${SOME_VAR}'"
Var is 'some value'
```

This is matching the behavior when `SOME_VAR` would be set within the shell expression by the user.
