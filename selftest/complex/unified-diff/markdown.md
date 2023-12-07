# Markdown Unified Diff

This test validates that unified Diffs of mismatching Markdown formatted tests can be re-applied to fix the test.

## Initial test fails

```scrut
$ "$SCRUT_BIN" test --renderer diff --match-markdown "*.mdtest" "$TESTDIR/invalid.mdtest" > output.diff
[50]
```

## Test output is valid unified diff format

```scrut
$ cat output.diff
--- *invalid.mdtest (glob)
+++ *invalid.mdtest.new (glob)
@@ -6 +6 @@ malformed output: This is the test
- not-bar
+ bar
@@ -15 +15 @@ malformed output: This is a test with test with correct quantifiers
-BAR
+bar
@@ -23 +23,3 @@ malformed output: This is a test with invalid quantifiers
-F* (glob+)
+foo
+foo
+foo
```

## Patch the invalid output

```scrut
$ cp "$TESTDIR/invalid.mdtest" invalid.mdtest && \
>   patch -o valid.mdtest invalid.mdtest < output.diff
patching file * (glob)
```

## Use the valid output as test

```scrut
$ "$SCRUT_BIN" test --renderer diff --match-markdown "*.mdtest" valid.mdtest
```
