Haz TESTDIR
  $ echo "TESTDIR: '$TESTDIR'"
  TESTDIR: '*selftest?cases' (glob)


Haz TESTFILE
  $ echo "TESTFILE: '$TESTFILE'"
  TESTFILE: 'environment.t'


Haz TMPDIR
  $ echo "TMPDIR: '$TMPDIR'"
  TMPDIR: '*execution.*__tmp' (glob)


Haz TESTSHELL
  $ echo "TESTSHELL: '$TESTSHELL'"
  TESTSHELL: '*bash*' (glob)


Haz LANG, LANGUAGE and LC_ALL
  $ echo "languages: '$LANG', '$LANGUAGE', '$LC_ALL'"
  languages: 'C', 'C', 'C'


Haz TZ
  $ echo "TZ: '$TZ'"
  TZ: 'GMT'


Haz COLUMNS
  $ echo "COLUMNS: '$COLUMNS'"
  COLUMNS: '80'


Haz CDPATH
  $ echo "CDPATH: '$CDPATH'"
  CDPATH: ''


Haz GREP_OPTIONS
  $ echo "GREP_OPTIONS: '$GREP_OPTIONS'"
  GREP_OPTIONS: ''


Haz GREP_OPTIONS
  $ echo "GREP_OPTIONS: '$GREP_OPTIONS'"
  GREP_OPTIONS: ''


Haz CRAMTMP
  $ echo "CRAMTMP: '$TMP'"
  CRAMTMP: '*execution.*' (glob)


Haz TMP
  $ echo "TMP: '$TMP'"
  TMP: '*execution.*__tmp' (glob)


Haz TEMP
  $ echo "TEMP: '$TEMP'"
  TEMP: '*execution.*__tmp' (glob)
