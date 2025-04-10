# Test the `create` command

Validate the generated output of the `create` command.

```scrut
$ alias scrut_run='"$SCRUT_BIN" create --log-level info --no-color'
```

## Create for STDOUT

````scrut
$ scrut_run --output - "echo Foo Bar Baz" 2>&1
* INFO scrut::utils::ui: ⭐️ Creating test for `echo Foo Bar Baz` (glob)
* INFO scrut::utils::ui: ✍️ STDOUT: Writing generated test document (glob)
# Command executes successfully

```scrut
$ echo Foo Bar Baz
Foo Bar Baz
```
````

## Create in file

````scrut
$ scrut_run --output "$TMPDIR"/out.md "echo Foo Bar Baz" 2>&1
* INFO scrut::utils::ui: ⭐️ Creating test for `echo Foo Bar Baz` (glob)
* INFO scrut::utils::ui: ✍️ *out.md: Writing generated test document (glob)
````

Validate created output

````scrut
$ cat "$TMPDIR"/out.md
# Command executes successfully

```scrut
$ echo Foo Bar Baz
Foo Bar Baz
```
````
