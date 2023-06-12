# CRLF output support

Scrut internally only uses LF (`\n`) line endings. At the point of ingestion (i.e. reading from a test file or reading output of execution) all CRLF (`\r\n`) are converted into `\n`.

This test file proves that Scrut can deal with CRLF execution output.

## Execution that outputs CRLF encoded lines work

```scrut
$ echo -ne 'foo\r\nbar\r\nbaz\r\n'
foo
bar
baz
```
