---
defaults:
   keep_crlf: true
   output_stream: combined
---

# Validate per-document defaults configuration

Tests in this file validate that the `defaults` set at per-document are used as defaults per-testcase

## Per-document defaults are used

```mooncram
$ echo -e "word\r"
word\r (escaped)
```

## Per-testcase overwrites per-document defaults

```mooncram {keep_crlf: false}
$ echo -e "word\r"
word
```

## Also test `output_stream`

```mooncram
$ echo a; echo b>&2
a
b
```

## Per-testcase overwrites per-document defaults, for `output_stream`

```mooncram {output_stream: stderr}
$ echo a; echo b>&2
b
```
