# Interactive Scrut Tests

Test if the mode setting in testcase configuration enables interactivity.

```scrut
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80   # Skip on Windows: PTY interactive
```

## A simple Scrut test that uses interactivity (long form notation)

```scrut {mode: interactive}
$ echo Hello && \
> echo -n "Give me input: " && \
> read -r SOME_VAR && \
> echo "Your input: $SOME_VAR" && \
> read -r -n 1 SOME_CHAR1 && \
> read -r -n 1 SOME_CHAR2 && \
> echo "Your input: $SOME_CHAR1 $SOME_CHAR2"
WAIT: Hello
WAIT: Give me input:
WRITE: Have some input
WAIT: Your input: Have some input
SEND_KEYS: a b
ASSERT: Your input: a b
```

## A simple Scrut test that uses interactivity (short form notation)

```scrut
% mode: interactive
$ echo Hello && \
> echo -n "Give me input: " && \
> read -r SOME_VAR && \
> echo "Your input: $SOME_VAR" && \
> read -r -n 1 SOME_CHAR1 && \
> read -r -n 1 SOME_CHAR2 && \
> echo "Your input: $SOME_CHAR1 $SOME_CHAR2"
@ Give me input:
| Have some input
@ Your input: Have some input
^ a b
@ Your input: a b
```
