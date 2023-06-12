CRLF must be accounted for
  $ echo -en 'Foo\r\nBar\r\nBaz\r\n'
  Foo\r (esc)
  Bar\r (esc)
  Baz\r (esc)
