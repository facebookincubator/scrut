# Newline handling

[Newline](https://en.wikipedia.org/wiki/Newline) endings is a sad story in computer history. In Unix / MacOS ( / \*BSD / Amiga / ...) the standard line ending is the line feed (LF) character `\n`. Microsoft DOS (also Palm OS and OS/2?) infamously attempted to make a combination of carriage return (CR) and line feed the standard: CRLF (`\r\n`). This made everybody mad - and they still are.

See the [`keep_crlf` configuration directive](/docs/reference/fundamentals/inline-configuration/) to understand how Scrut handles LF and CRLF and how you can modify the default behavior.
