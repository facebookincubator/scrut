# Interactive terminal cursor reports

The application asks its terminal for the cursor position before accepting user
input. Scrut answers the protocol query; the test supplies only user interaction.

```scrut
$ [[ "$(uname)" == "Linux" ]] || [[ "$(uname)" == "Darwin" ]] || exit 80
```

```scrut {mode: interactive}
$ bash "$TESTDIR/cursor-report.sh"
WAIT: Name:
WRITE: Alice
WAIT: Hello Alice
```
