# Cram Unified Diff

This test validates that unified Diffs of mismatching Cram formatted tests can be re-applied to fix the test.

## Initial test fails

```scrut
$ "$SCRUT_BIN" test --renderer diff --match-cram "*.cramtest" "$TESTDIR/invalid.cramtest" > output.diff
[50]
```

## Test output is valid unified diff format

```scrut
$ cat output.diff
--- *invalid.cramtest (glob)
+++ *invalid.cramtest.new (glob)
@@ -4 +4 @@ malformed output: This is the test
-   not-bar
+   bar
@@ -10 +10 @@ malformed output: This is a test with test with correct quantifiers
-  BAR
+  bar
@@ -15 +15,3 @@ malformed output: This is a test with invalid quantifiers
-  F* (glob+)
+  foo
+  foo
+  foo
```

## Patch the invalid output

```scrut
$ cp "$TESTDIR/invalid.cramtest" invalid.cramtest && \
>   patch -o valid.cramtest invalid.cramtest < output.diff
patching file * (glob)
```

## Use the valid output as test

```scrut
$ "$SCRUT_BIN" test --renderer diff --match-cram "*.cramtest" valid.cramtest
```
