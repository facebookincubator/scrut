# Shell Completions

## Bootstrap

```mooncram
$ . "${TESTDIR}/setup.sh"
OK
```

## Bash completions define function and include subcommands

```mooncram
$ _MOON_CRAM_COMPLETE=bash_source "${MOON_CRAM_BIN}" | grep -E "^_moon-cram\(\)|cmd=\"moon__cram__subcmd__(create|test|update)\""
_moon-cram() {
                cmd="moon__cram__subcmd__create"
                cmd="moon__cram__subcmd__test"
                cmd="moon__cram__subcmd__update"
```

## Zsh completions include compdef and subcommand descriptions

```mooncram
$ _MOON_CRAM_COMPLETE=zsh_source "${MOON_CRAM_BIN}" | grep -E "^#compdef|'(create|test|update):" | head -4
#compdef moon-cram
'create:Create tests from provided shell expression' \
'test:Run tests from files or directories' \
'update:Re-run all testcases in given file(s) and update the output expectations' \
```

## Fish completions include complete commands for moon-cram

```mooncram
$ _MOON_CRAM_COMPLETE=fish_source "${MOON_CRAM_BIN}" | grep "^complete -c moon-cram" | head -3
complete -c moon-cram * (glob+)
```

## PowerShell completions include Register-ArgumentCompleter and subcommands

```mooncram
$ _MOON_CRAM_COMPLETE=powershell_source "${MOON_CRAM_BIN}" | grep -E "Register-ArgumentCompleter|CompletionResult.*'(create|test|update)'" | head -4
Register-ArgumentCompleter -Native -CommandName 'moon-cram' -ScriptBlock {
            [CompletionResult]::new('create', 'create', [CompletionResultType]::ParameterValue, 'Create tests from provided shell expression')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Run tests from files or directories')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Re-run all testcases in given file(s) and update the output expectations')
```

## Elvish completions include edit:completion setup and subcommands

```mooncram
$ _MOON_CRAM_COMPLETE=elvish_source "${MOON_CRAM_BIN}" | grep -E "edit:completion:arg-completer|cand (create|test|update)" | head -4
set edit:completion:arg-completer[moon-cram] = {|@words|
            cand create 'Create tests from provided shell expression'
            cand test 'Run tests from files or directories'
            cand update 'Re-run all testcases in given file(s) and update the output expectations'
```

## Invalid completion value shows error

```mooncram
$ _MOON_CRAM_COMPLETE=invalid "${MOON_CRAM_BIN}" 2>&1
Error: Invalid value passed to environment variable '_MOON_CRAM_COMPLETE'
Valid values: bash_source, elvish_source, fish_source, powershell_source, zsh_source (?)
[1]
```

## Normal operation unaffected when env var not set

```mooncram
$ "${MOON_CRAM_BIN}" --help | head -5
A testing toolkit for CLI applications

Usage: moon-cram(?:\.exe)? .* (regex)

Commands:
```
