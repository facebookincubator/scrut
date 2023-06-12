# Here is a CRLF encoded file

Scrut internally only uses LF (`\n`) line endings. At the point of ingestion (i.e. reading from a test file or reading output of execution) all CRLF (`\r\n`) are converted into `\n`.

This test file proves that Scrut can deal with CRLF encoded test files.

## Test from within CRLF encoded file works

```scrut
$ echo foo
foo
```
