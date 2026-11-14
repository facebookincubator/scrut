# Update preserves multiline testcase configuration

```scrut
$ cp "$TESTDIR"/*.mdtest .
```

Update a document containing configuration, comments and outdated expectations:

```scrut
$ "$SCRUT_BIN" update --replace --assume-yes --match-markdown '*.mdtest' config.mdtest
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

Only the outdated expectations change, including output resembling configuration:

```scrut
$ sed -e 's/^old$/new/' -e 's/^% output_stream: old$/% output_stream: stdout/' -e 's/^# old output comment$/# output comment/' "$TESTDIR/config.mdtest" > expected.mdtest; diff expected.mdtest config.mdtest
```

The updated document still passes:

```scrut
$ "$SCRUT_BIN" test --match-markdown '*.mdtest' config.mdtest
Result: 1 document(s) with 3 testcase(s): 3 succeeded, 0 failed and 0 skipped
```

Updating the passing document leaves it unchanged:

```scrut
$ "$SCRUT_BIN" update --replace --assume-yes --match-markdown '*.mdtest' config.mdtest
Result: 1 document(s) of which 0 updated, 0 skipped and 1 unchanged
```

Pre-command blank and ordinary expectations remain expectations:

```scrut
$ "$SCRUT_BIN" test --match-markdown '*.mdtest' pre-command.mdtest
Result: 1 document(s) with 3 testcase(s): 3 succeeded, 0 failed and 0 skipped
```

```scrut
$ "$SCRUT_BIN" update --replace --assume-yes --match-markdown '*.mdtest' pre-command.mdtest
Result: 1 document(s) of which 1 updated, 0 skipped and 0 unchanged
```

```scrut
$ "$SCRUT_BIN" test --match-markdown '*.mdtest' pre-command.mdtest
Result: 1 document(s) with 3 testcase(s): 3 succeeded, 0 failed and 0 skipped
```

```scrut
$ "$SCRUT_BIN" update --replace --assume-yes --match-markdown '*.mdtest' pre-command.mdtest
Result: 1 document(s) of which 0 updated, 0 skipped and 1 unchanged
```
