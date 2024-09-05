A simple string without new line
  $ echo -n hello
  hello (no-eol)


A simple string with newline
  $ echo this should be working
  this should be working


Using expanding regular expressions
  $ echo -ne "foo is\nbar1\nbar2\nbar3\nbaz"
  foo is
  bar\d+ (re+)
  baz (no-eol)


Using expanding globs
  $ echo -e "foo is\nbar1\nbar2\nbar3\nbaz"
  foo is
  bar* (glob+)
  baz


Setting shell state
  $ shopt -s expand_aliases
  > SOME_VAR1=foo1
  > export SOME_VAR2=foo2
  > some_function() {
  >   echo foo3
  > }
  > alias some_alias='echo foo4'


Using shell state
  $ echo "shell var: $SOME_VAR1"
  > echo "env var:   $SOME_VAR2"
  > echo "function:  $(some_function)"
  > echo "alias:     $(some_alias)"
  shell var: foo1
  env var:   foo2
  function:  foo3
  alias:     foo4
