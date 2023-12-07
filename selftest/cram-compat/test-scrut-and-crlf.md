# Scrut doesn't care about CRLF

```scrut
$ echo -en 'Foo\r\nBar\r\nBaz\r\n'
Foo
Bar
Baz
```
