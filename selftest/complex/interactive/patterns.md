# Interactive Mode: Pattern Matching

Tests for WAIT and ASSERT with regex and glob patterns.

```scrut
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80   # Skip on Windows: PTY interactive
```

## Wait with regex pattern

```scrut {mode: interactive}
$ echo "Version 1.2.3 loaded"
WAIT: Version [0-9]+\.[0-9]+\.[0-9]+ loaded (regex)
```

## Wait with glob pattern

```scrut {mode: interactive}
$ echo "Config file: /etc/app/config.yaml"
WAIT: Config file: * (glob)
```

## Assert with regex

```scrut {mode: interactive}
$ echo "Total: 42 items"
@ Total:
ASSERT: Total: [0-9]+ items (regex)
```
