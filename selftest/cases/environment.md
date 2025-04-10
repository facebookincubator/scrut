# Environment in Scrut tests

## Haz TESTDIR

```scrut
$ echo "TESTDIR: '$TESTDIR'"
TESTDIR: '*selftest?cases' (glob)
```

## Haz TESTFILE

```scrut
$ echo "TESTFILE: '$TESTFILE'"
TESTFILE: 'environment.md'
```

## Haz TMPDIR

```scrut
$ echo "TMPDIR: '$TMPDIR'"
TMPDIR: '*__tmp' (glob)
```

## Haz TESTSHELL

```scrut
$ echo "TESTSHELL: '$TESTSHELL'"
TESTSHELL: '*bash*' (glob)
```

## Haz LANG, LANGUAGE and LC_ALL

```scrut
$ echo "languages: '$LANG', '$LANGUAGE', '$LC_ALL'"
languages: 'C', 'C', 'C'
```

## Haz TZ

```scrut
$ echo "TZ: '$TZ'"
TZ: 'GMT'
```

## Haz COLUMNS

```scrut
$ echo "COLUMNS: '$COLUMNS'"
COLUMNS: '80'
```

## Haz CDPATH

```scrut
$ echo "CDPATH: '$CDPATH'"
CDPATH: ''
```

## Haz GREP_OPTIONS

```scrut
$ echo "GREP_OPTIONS: '$GREP_OPTIONS'"
GREP_OPTIONS: ''
```

## Haz GREP_OPTIONS

```scrut
$ echo "GREP_OPTIONS: '$GREP_OPTIONS'"
GREP_OPTIONS: ''
```

## Haz SCRUT_TEST

```scrut
$ echo "SCRUT_TEST: $SCRUT_TEST"
SCRUT_TEST: .*selftest[/\\]cases[/\\]environment\.md:76 (regex)
```
