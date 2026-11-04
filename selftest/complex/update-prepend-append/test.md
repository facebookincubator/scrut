# Update skips documents that use prepend or append

Tests in this file validate that `update` leaves documents with `prepend` or `append` in their front matter unchanged, because it does not run the prepended or appended files.

```scrut
$ alias scrut_update='$SCRUT_BIN update --match-markdown="*.mdtest"'
```

## Copy documents to update

```scrut
$ cp "$TESTDIR"/*.mdtest .
```

## Document with `prepend` is skipped

```scrut
$ scrut_update --replace --assume-yes prepend.mdtest
Result: 1 document(s) of which 0 updated, 1 skipped and 0 unchanged
```

```scrut
$ diff "$TESTDIR"/prepend.mdtest prepend.mdtest
```

## Document with `append` is skipped

```scrut
$ scrut_update --replace --assume-yes append.mdtest
Result: 1 document(s) of which 0 updated, 1 skipped and 0 unchanged
```

```scrut
$ diff "$TESTDIR"/append.mdtest append.mdtest
```
