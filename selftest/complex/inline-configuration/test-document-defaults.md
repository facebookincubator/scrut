---
defaults:
   keep_crlf: true
---

# Validate per-document defaults configuration

Tests in this file validate that the `defaults` set at per-document are used as defaults per-testcase

## Per-document defaults are used

```scrut
$ echo -e "word\r"
word\r (escaped)
```

## Per-testcase overwrites per-document dfaults

```scrut {keep_crlf: false}
$ echo -e "word\r"
word
```
