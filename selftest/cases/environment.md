# Environment in Moon Cram tests

## Haz TESTDIR

```mooncram
$ echo "TESTDIR: '$TESTDIR'"
TESTDIR: '*selftest?cases' (glob)
```

## Haz TESTFILE

```mooncram
$ echo "TESTFILE: '$TESTFILE'"
TESTFILE: 'environment.md'
```

## Haz TMPDIR

```mooncram
$ echo "TMPDIR: '$TMPDIR'"
TMPDIR: '*__tmp' (glob)
```

## Haz TESTSHELL

```mooncram
$ echo "TESTSHELL: '$TESTSHELL'"
TESTSHELL: '*bash*' (glob)
```

## Haz LANG, LANGUAGE and LC_ALL

```mooncram
$ echo "languages: '$LANG', '$LANGUAGE', '$LC_ALL'"
languages: 'C', 'C', 'C'
```

## Haz TZ

```mooncram
$ echo "TZ: '$TZ'"
TZ: 'GMT'
```

## Haz COLUMNS

```mooncram
$ echo "COLUMNS: '$COLUMNS'"
COLUMNS: '80'
```

## Haz CDPATH

```mooncram
$ echo "CDPATH: '$CDPATH'"
CDPATH: ''
```

## Haz GREP_OPTIONS

```mooncram
$ echo "GREP_OPTIONS: '$GREP_OPTIONS'"
GREP_OPTIONS: ''
```

## Haz GREP_OPTIONS

```mooncram
$ echo "GREP_OPTIONS: '$GREP_OPTIONS'"
GREP_OPTIONS: ''
```

## Haz MOON_CRAM_TEST

```mooncram
$ echo "MOON_CRAM_TEST: $MOON_CRAM_TEST"
MOON_CRAM_TEST: .*selftest[/\\]cases[/\\]environment\.md:76 (regex)
```
