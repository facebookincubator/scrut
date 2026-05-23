# Validate per-testcase environment configuration

Tests in this file validate that the `environment` configuration sets environment variables as intended.

## Environment variable is not set initially

```mooncram
$ echo "Var is '${SOME_VAR}'"
Var is ''
```

## Environment variable is set

```mooncram {environment: {"SOME_VAR": "some value"}}
$ echo "Var is '${SOME_VAR}'"
Var is 'some value'
```

## Environment variable will not be unset

```mooncram
$ echo "Var is '${SOME_VAR}'"
Var is 'some value'
```

This is matching the behavior when `SOME_VAR` would be set within the shell expression by the user.
