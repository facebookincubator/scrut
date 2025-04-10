import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Test Creation

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

As previously decided, the first test will validate that `jq --version` executes successfully. Running this command should produce output similar to the following (your version may vary):

```bash title="Terminal"
$ jq --version
jq-1.7.1
```

## Using Scrut's Built-in Test Creation

Generating a Scrut test from the command line is pretty straight forward:

```bash title="Terminal"
$ scrut create --output tests/smoke.md -- jq --version
✍️ /tmp/smoke.md: Writing generated test document
```

This will create a test file that should look like this:

````markdown title="tests/smoke.md"
# Command executes successfully

```scrut
$ jq --version
jq-1.7.1
```
````

You can now execute the newly created test file with:

```bash title="Terminal"
$ scrut test tests/smoke.md
🔎 Found 1 test document(s)

Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

:::note

The `scrut test` command accepts arbitrary files or directories. All of the following (assuming the paths exist) are valid:
- `scrut test tests` - test every test file found (recursively) in `tests`
- `scrut test tests/smoke.md tests/other.md` - test both files `tests/smoke.md` and `tests/other.md`
- `scrut test tests other-tests` - test all files found (recursively) in the `tests` and `other-tests` directories

:::

### Use STDIN to receive commands

Alternatively you can also pipe the command via STDIN to scrut create:

```bash title="Terminal"
$ echo "jq --version" | scrut create - > tests/smoke.md
✍️ STDOUT: Writing generated test document
```

Here also `--output` was omitted, in which case `scrut create` will print the newly created test file to STDOUT. Check out `scrut create --help` to see all options.

## Write tests manually

You can of course also create your `tests/smoke.md` file manually in a text editor. As Scrut test documents are written in Markdown any Markdown syntax highlighting plugin for your IDE of choice will help greatly.
