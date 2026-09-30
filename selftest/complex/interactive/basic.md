# Interactive Mode: Basic Tests

Tests for the interactive validation mode using simple `read` + `echo` patterns.

```scrut
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80   # Skip on Windows: PTY interactive
```

## Basic read and echo (long form)

```scrut {mode: interactive}
$ bash -c 'read -p "Name: " name && echo "Hello $name"'
WAIT: Name:
WRITE: test_user
WAIT: Hello test_user
```

## Basic read and echo (short form)

```scrut {mode: interactive}
$ bash -c 'read -p "Flavor: " flavor && echo "You chose $flavor"'
@ Flavor:
| vanilla
@ You chose vanilla
```

## Assert on output (long form)

```scrut {mode: interactive}
$ echo "Welcome to the app"
WAIT: Welcome
ASSERT: Welcome to the app
```

## Assert on output (short form)

```scrut {mode: interactive}
$ echo "Hello World"
@ Hello
! Hello World
```
