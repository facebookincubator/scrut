This is only relevant for Window
  $ [[ $(uname -s) =~ ^(MINGW64|CYGWIN)_NT ]] || exit 80

TEMP comes with a non-empty value
  $ echo "TEMP = $TEMP"
  TEMP = *\execution.*\__tmp (glob)

Set TEMP environment variable
  $ export TEMP=somevalue

TEMP environment variable retains user-defined value
  $ echo "TEMP = $TEMP"
  TEMP = somevalue

TMP comes with a non-empty value
  $ echo "TMP = $TMP"
  TMP = *\execution.*\__tmp (glob)

Set TMP environment variable
  $ export TMP=somevalue

TMP environment variable retains user-defined value
  $ echo "TMP = $TMP"
  TMP = somevalue
