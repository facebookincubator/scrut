# Test whether standard environment variables are available

See: https://www.internalfb.com/intern/wiki/CLI_Foundation/Tools/Scrut/Advanced/Specifics/#common-linux-environment

## Common (linux) environment variables

```scrut
$ echo "LANG = '$LANG'"
> echo "LANGUAGE = '$LANGUAGE'"
> echo "LC_ALL = '$LC_ALL'"
> echo "TZ = '$TZ'"
> echo "COLUMNS = '$COLUMNS'"
> echo "CDPATH = '$CDPATH'"
> echo "GREP_OPTIONS = '$GREP_OPTIONS'"
LANG = 'C'
LANGUAGE = 'C'
LC_ALL = 'C'
TZ = 'GMT'
COLUMNS = '80'
CDPATH = ''
GREP_OPTIONS = ''
```
