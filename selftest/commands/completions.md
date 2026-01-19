# Shell Completions

## Bootstrap

```scrut
$ . "${TESTDIR}/setup.sh"
OK
```

## Bash completions define function and include subcommands

```scrut
$ _SCRUT_COMPLETE=bash_source "${SCRUT_BIN}" | grep -E "^_scrut\(\)|scrut__(create|test|update)"
_scrut() {
                cmd="scrut__create"
                cmd="scrut__test"
                cmd="scrut__update"
        scrut__create)
        scrut__test)
        scrut__update)
```

## Zsh completions include compdef and subcommand descriptions

```scrut
$ _SCRUT_COMPLETE=zsh_source "${SCRUT_BIN}" | grep -E "^#compdef|'(create|test|update):" | head -4
#compdef scrut
'create:Create tests from provided shell expression' \
'test:Run tests from files or directories' \
'update:Re-run all testcases in given file(s) and update the output expectations' \
```

## Fish completions include complete commands for scrut

```scrut
$ _SCRUT_COMPLETE=fish_source "${SCRUT_BIN}" | grep "^complete -c scrut" | head -3
complete -c scrut * (glob+)
```

## Invalid completion value shows error

```scrut
$ _SCRUT_COMPLETE=invalid "${SCRUT_BIN}" 2>&1
Error: Invalid value for _SCRUT_COMPLETE: 'invalid'
Valid values: bash_source, elvish_source, fish_source, powershell_source, zsh_source
[1]
```

## Normal operation unaffected when env var not set

```scrut
$ "${SCRUT_BIN}" --help | head -5
A testing toolkit to scrutinize CLI applications

Usage: scrut * (glob)

Commands:
```
