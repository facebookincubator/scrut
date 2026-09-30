# Interactive Mode: Multiline Config

Tests for interactive mode using `%` multiline config syntax.

```scrut
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80   # Skip on Windows: PTY interactive
```

## Mode set via multiline config

```scrut
% mode: interactive
$ bash -c 'read -p "Color: " color && echo "Selected $color"'
@ Color:
| blue
@ Selected blue
```
