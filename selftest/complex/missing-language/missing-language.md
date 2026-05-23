# Error on code block with no language specified

```mooncram
$ $MOON_CRAM_BIN test --match-markdown "*.mdtest" "$TESTDIR"/missing-language.mdtest 2>&1
* Failed to parse test from "*missing-language.mdtest" with markdown parser (glob)

Caused by:
    Code block starting at line 2 is missing language specifier. Use ```mooncram to make this block a Moon Cram test, or any other language to make Moon Cram skip this block.
[1]
```

# Error on EMPTY code block with no language specified

```mooncram
$ $MOON_CRAM_BIN test --match-markdown "*.mdtest" "$TESTDIR"/missing-language-empty-block.mdtest 2>&1
* Failed to parse test from "*missing-language-empty-block.mdtest" with markdown parser (glob)

Caused by:
    Code block starting at line 2 is missing language specifier. Use ```mooncram to make this block a Moon Cram test, or any other language to make Moon Cram skip this block.
[1]
```
