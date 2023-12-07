# Markdown Unified Diff

This test validates that unified diff outputs that change multiple files are valid and can be applied.

## Copy files to current temp path

```scrut
$ cp "$TESTDIR"/multi*.mdtest .
```

## Initial test of multiple files fails

```scrut
$ "$SCRUT_BIN" test --renderer diff --match-markdown "*.mdtest" . > output.diff
[50]
```

## Test output is valid unified diff format

```scrut
$ cat output.diff
--- \.[/\\]multi-test-1\.mdtest (regex)
\+\+\+ \.[/\\]multi-test-1\.mdtest\.new (regex)
@@ -6 +6 @@ malformed output: This is the test
- not-bar
+ bar

--- \.[/\\]multi-test-2\.mdtest (regex)
\+\+\+ \.[/\\]multi-test-2\.mdtest\.new (regex)
@@ -6 +6 @@ malformed output: This is the test 1
- NOT-BAR
+ BAR
@@ -15 +15 @@ malformed output: This is the test 2
- NOT-BBB
+ BBB
```

## Patch the invalid output

```scrut
$ patch < output.diff
patching file * (glob+)
```

## Use the valid output as test

```scrut
$ "$SCRUT_BIN" test --renderer diff --match-markdown "*.mdtest" .
```
