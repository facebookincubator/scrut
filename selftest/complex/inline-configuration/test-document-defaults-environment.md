---
defaults:
  environment:
    FOO: default
    BAR: default
---

# Validate per-document defaults for `environment`

Tests in this file validate that a per-testcase `environment` is merged with the per-document `defaults`, and that the per-testcase value wins for the same variable.

## Per-testcase overwrites per-document defaults, for `environment`

```scrut {environment: {FOO: testcase}}
$ echo "FOO=$FOO BAR=$BAR"
FOO=testcase BAR=default
```
