# Test inline setup / inline bootstrapping

Each test file should be concerned with one use-case -- or any other contained context that makes sense. Because of this there are usually multiple test files and chances are high that, if bootstrapping is needed, they all need the same bootstrapping. Hence the best practice recommendation is to outsource bootstrapping into a dedicated file (e.g. `setup.sh`) -- or multiple files, if need be.

However, in edge-cases where only one test file exists -- or bootstrap code is not shared -- it makes sense to inline the bootstrapping, which likely consists of the execution of _multiple_ commands.

This test show-cases a practice that works for such cases.

## Run bootstrap block of multiple commands

```scrut
$ shopt -s expand_aliases && \
>   alias foo="echo Foo" && \
>   alias bar="echo Bar" && \
>   alias baz="foo ; bar" && \
>   export SOMETHING=whatever
```

## Assert aliases are exported

```scrut
$ bar && foo
Bar
Foo
```

## Assure aliases of aliases are exported

```scrut
$ baz
Foo
Bar
```

## Assure environment variables are exported

```scrut
$ echo "Say ${SOMETHING}"
Say whatever
```
