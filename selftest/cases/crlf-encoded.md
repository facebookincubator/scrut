# Here is a CRLF encoded file

Moon Cram internally only uses LF (`\n`) line endings. At the point of ingestion (i.e. reading from a test file or reading output of execution) all CRLF (`\r\n`) are converted into `\n`.

This test file proves that Moon Cram can deal with CRLF encoded test files.

## Test from within CRLF encoded file works

```mooncram
$ echo foo
foo
```
