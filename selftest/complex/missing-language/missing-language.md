# Error on code block with no language specified

```scrut
$ $SCRUT_BIN test --match-markdown "*.mdtest" "$TESTDIR"/missing-language.mdtest 2>&1
* parse test from "*missing-language.mdtest" with markdown parser (glob)

Caused by:
    Code block starting at line 2 is missing language specifier. Use ```scrut to make this block a Scrut test, or any other language to make Scrut skip this block.
* (glob?)
[1]
```

# Error on EMPTY code block with no language specified

```scrut
$ $SCRUT_BIN test --match-markdown "*.mdtest" "$TESTDIR"/missing-language-empty-block.mdtest 2>&1
* parse test from "*missing-language-empty-block.mdtest" with markdown parser (glob)

Caused by:
    Code block starting at line 2 is missing language specifier. Use ```scrut to make this block a Scrut test, or any other language to make Scrut skip this block.
* (glob?)
[1]
```
