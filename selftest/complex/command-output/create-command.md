# Test the `create` command

Validate the generated output of the `create` command.

```mooncram
$ alias moon_cram_run='"$MOON_CRAM_BIN" create --log-level info --no-color'
```

## Create for STDOUT

````mooncram
$ moon_cram_run --output - "echo Foo Bar Baz" 2>&1
* INFO moon_cram::utils::ui: ⭐️ Creating test for `echo Foo Bar Baz` (glob)
* INFO moon_cram::utils::ui: ✍️ STDOUT: Writing generated test document (glob)
# Command executes successfully

```mooncram
$ echo Foo Bar Baz
Foo Bar Baz
```
````

## Create in file

````mooncram
$ moon_cram_run --output "$TMPDIR"/out.md "echo Foo Bar Baz" 2>&1
* INFO moon_cram::utils::ui: ⭐️ Creating test for `echo Foo Bar Baz` (glob)
* INFO moon_cram::utils::ui: ✍️ *out.md: Writing generated test document (glob)
````

Validate created output

````mooncram
$ cat "$TMPDIR"/out.md
# Command executes successfully

```mooncram
$ echo Foo Bar Baz
Foo Bar Baz
```
````
