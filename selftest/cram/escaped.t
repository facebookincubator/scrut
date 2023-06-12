Escaped color code
  $ echo -e "foo \033[1mbar\033[0m baz"
  foo \x1b[1mbar\x1b[0m baz (escaped)


Escaped color code shorthand
  $ echo -e "foo \033[1mbar\033[0m baz"
  foo \e[1mbar\e[0m baz (escaped)


Escape unprintable character
  $ echo -e 'foo\003 bar'
  foo\x03 bar (escaped)


Double escapes are fine
  $ echo -e 'foo\\nbar'
  foo\\nbar (escaped)


Escaped Alarm Bell
  $ echo -e 'foo\abar\abaz'
  foo\abar\abaz (escaped)


Escaped Overprint
  $ echo -e 'foo\bbar\bbaz'
  foo\bbar\bbaz (escaped)


Escaped Form Feed
  $ echo -e 'foo\fbar\fbaz'
  foo\fbar\fbaz (escaped)


Escaped Carriage Return (CR)
  $ echo -e 'foo\rbar\rbaz'
  foo\rbar\rbaz (escaped)


Escaped Horizontal Tab
  $ echo -e "foo\tbar\tbaz"
  foo\tbar\tbaz (escaped)


Escaped Vertical Tab
  $ echo -e 'foo\vbar\vbaz'
  foo\vbar\vbaz (escaped)


Don't escape unicode
  $ echo -e 'foo 😁\n🦀 bar\nbaz 🤡'
  foo \xf0\x9f\x98\x81 (escaped)
  \xf0\x9f\xa6\x80 bar (escaped)
  baz \xf0\x9f\xa4\xa1 (escaped)
