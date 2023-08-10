In order to transfer state in between executions in two different `bash` processes, variables
(among other things) must be exported *after* the previous execution and imported *before* the next
execution.

To get all variables the `declare -p` built-in is used after the execution to print all currently
set shell and environment variables.
However, not all variables this prints can also be imported in a subsequent execution:
- Some variables are read-only and an attempt to import them in a subsequent execution would result
  in a failure (exit != 0) and printing of a message on STDERR.
- Other variables are maintained by bash itself, or other built-ins, and modifying them would
  interfere with that.

Following a list of all `man bash` [named variables](https://www.man7.org/linux/man-pages/man1/bash.1.html)
(as of version 5.2) and whether they are included (`INCL`) or excluded (`*EXCL*`) when persisting
the state:

| VARIABLE NAME           | EXPORT   | NOTE                                                            |
| ----------------------- | -------- | --------------------------------------------------------------- |
| `BASH`                  | INCL     | if set (differently), then intentionally by user                |
| `BASHOPTS`              | **EXCL** | must be ignored, would interfere with `shopt` import            |
| `BASHPID`               | INCL     | assignment has no effect                                        |
| `BASH_ALIASES`          | **EXCL** | must be ignored, would interfere with `alias` import            |
| `BASH_ARGC`             | **EXCL** | may result in inconsistent values (man)                         |
| `BASH_ARGV`             | **EXCL** | may result in inconsistent values (man)                         |
| `BASH_ARGV0`            | **EXCL** | may result in inconsistent values (man)                         |
| `BASH_CMDS`             | **EXCL** | bash-owned                                                      |
| `BASH_COMMAND`          | **EXCL** | bash-owned                                                      |
| `BASH_EXECUTION_STRING` | **EXCL** | bash-owned                                                      |
| `BASH_LINENO`           | **EXCL** | bash-owned                                                      |
| `BASH_LOADABLES_PATH`   | INCL     | user-owned                                                      |
| `BASH_REMATCH`          | **EXCL** | bash-owned                                                      |
| `BASH_SOURCE`           | **EXCL** | bash-owned                                                      |
| `BASH_SUBSHELL`         | **EXCL** | bash-owned                                                      |
| `BASH_VERSINFO`         | **EXCL** | bash-owned                                                      |
| `BASH_VERSION`          | INCL     | safe to tinker with                                             |
| `COMP_CWORD`            | INCL     | user-owned                                                      |
| `COMP_KEY`              | INCL     | user-owned                                                      |
| `COMP_LINE`             | INCL     | user-owned                                                      |
| `COMP_POINT`            | INCL     | user-owned                                                      |
| `COMP_TYPE`             | INCL     | user-owned                                                      |
| `COMP_WORDBREAKS`       | INCL     | user-owned                                                      |
| `COMP_WORDS`            | INCL     | user-owned                                                      |
| `COPROC`                | **EXCL** | would interfere with `coproc` execution                         |
| `DIRSTACK`              | **EXCL** | interferes with `pushd` / `popd` import, will be rebuild anyway |
| `EPOCHREALTIME`         | INCL     | assignments are ignored                                         |
| `EPOCHSECONDS`          | INCL     | assignments are ignored                                         |
| `EUID`                  | **EXCL** | read-only                                                       |
| `FUNCNAME`              | **EXCL** | interferes with function export                                 |
| `GROUPS`                | INCL     | assignments are ignored                                         |
| `HISTCMD`               | INCL     | assignments are ignored                                         |
| `HOSTNAME`              | INCL     | user-owned                                                      |
| `HOSTTYPE`              | INCL     | user-owned                                                      |
| `LINENO`                | **EXCL** | may result in inconsistent values                               |
| `MACHTYPE`              | INCL     | user-owned                                                      |
| `MAPFILE`               | INCL     | user-owned                                                      |
| `OLDPWD`                | INCL     | user-owned                                                      |
| `OPTARG`                | INCL     | user-owned                                                      |
| `OPTIND`                | INCL     | user-owned                                                      |
| `OSTYPE`                | INCL     | user-owned                                                      |
| `PIPESTATUS`            | INCL     | assignments are ignored                                         |
| `PPID`                  | **EXCL** | read-only                                                       |
| `PWD`                   | INCL     | set by last `cd`                                                |
| `RANDOM`                | INCL     | sets a rand seed                                                |
| `READLINE_ARGUMENT`     | INCL     | bind stuff                                                      |
| `READLINE_LINE`         | INCL     | bind stuff                                                      |
| `READLINE_MARK`         | INCL     | bind stuff                                                      |
| `READLINE_POINT`        | INCL     | bind stuff                                                      |
| `REPLY`                 | INCL     | is overwritten by read                                          |
| `SECONDS`               | INCL     | write resets it, will reset anyway                              |
| `SHELLOPTS`             | **EXCL** | interferes with `set` export                                    |
| `SHLVL`                 | INCL     | would be set to the same value                                  |
| `SRANDOM`               | INCL     | just seeds it for random                                        |
| `UID`                   | **EXCL** | read-only                                                       |
| `BASH_COMPAT`           | INCL     | user-owned                                                      |
| `BASH_ENV`              | INCL     | user-owned                                                      |
| `BASH_XTRACEFD`         | INCL     | can be set to valid file descriptor                             |
| `CDPATH`                | INCL     | user-owned                                                      |
| `CHILD_MAX`             | INCL     | user-owned                                                      |
| `COLUMNS`               | INCL     | user-owned                                                      |
| `COMPREPLY`             | INCL     | user-owned                                                      |
| `EMACS`                 | INCL     | user-owned                                                      |
| `ENV`                   | INCL     | user-owned                                                      |
| `EXECIGNORE`            | INCL     | user-owned                                                      |
| `FCEDIT`                | INCL     | user-owned                                                      |
| `FIGNORE`               | INCL     | user-owned                                                      |
| `FUNCNEST`              | INCL     | user-owned                                                      |
| `GLOBIGNORE`            | INCL     | user-owned                                                      |
| `HISTCONTROL`           | INCL     | user-owned                                                      |
| `HISTFILE`              | INCL     | user-owned                                                      |
| `HISTFILESIZE`          | INCL     | user-owned                                                      |
| `HISTIGNORE`            | INCL     | user-owned                                                      |
| `HISTSIZE`              | INCL     | user-owned                                                      |
| `HISTTIMEFORMAT`        | INCL     | user-owned                                                      |
| `HOME`                  | INCL     | user-owned                                                      |
| `HOSTFILE`              | INCL     | user-owned                                                      |
| `IFS`                   | INCL     | user-owned                                                      |
| `IGNOREEOF`             | INCL     | user-owned                                                      |
| `INPUTRC`               | INCL     | user-owned                                                      |
| `INSIDE_EMACS`          | INCL     | user-owned                                                      |
| `LANG`                  | INCL     | user-owned                                                      |
| `LC_ALL`                | INCL     | user-owned                                                      |
| `LC_COLLATE`            | INCL     | user-owned                                                      |
| `LC_CTYPE`              | INCL     | user-owned                                                      |
| `LC_MESSAGES`           | INCL     | user-owned                                                      |
| `LC_NUMERIC`            | INCL     | user-owned                                                      |
| `LC_TIME`               | INCL     | user-owned                                                      |
| `LINES`                 | INCL     | user-owned                                                      |
| `MAIL`                  | INCL     | user-owned                                                      |
| `MAILCHECK`             | INCL     | user-owned                                                      |
| `MAILPATH`              | INCL     | user-owned                                                      |
| `OPTERR`                | INCL     | user-owned                                                      |
| `PATH`                  | INCL     | user-owned                                                      |
| `POSIXLY_CORRECT`       | INCL     | user-owned                                                      |
| `PROMPT_COMMAND`        | INCL     | user-owned                                                      |
| `PROMPT_DIRTRIM`        | INCL     | user-owned                                                      |
| `PS0`                   | INCL     | user-owned                                                      |
| `PS1`                   | INCL     | user-owned                                                      |
| `PS2`                   | INCL     | user-owned                                                      |
| `PS3`                   | INCL     | user-owned                                                      |
| `PS4`                   | INCL     | user-owned                                                      |
| `SHELL`                 | INCL     | user-owned                                                      |
| `TIMEFORMAT`            | INCL     | user-owned                                                      |
| `TMOUT`                 | INCL     | user-owned                                                      |
| `TMPDIR`                | INCL     | user-owned                                                      |
| `auto_resume`           | INCL     | user-owned                                                      |
| `histchars`             | INCL     | user-owned                                                      |

With:

- *user-owned* refers to variables that, if they are set or changed then by the user either directly
  (e.g. `PS1`) or indirectly (e.g. `EMACS`) with good cause and hence must be part of the state of
  the next execution.
- *bash-owned* refers to variables that are managed by `bash` itself and should not be modified,
  even by the export / import in between executions, or it may confuse `bash` itself.
- *read-only* variables cannot be set and any attempt to do so results in failure and a message on
  STDERR.
  These messages would interfere with the capturing of the STDERR output by Scrut and the failure
  would trigger if `set -e`.
  Hence read-only variables must be excluded from export, so not to cause those errors on import.
  Variables are exported using the built-in `declare -p`.
  In newer `bash` versions this lists all variables with a prefixed declare statement, like
  `delare -- VARNAME=varvalue` and notes if the variable is read-only (e.g.
  `declare -r VARNAME=varvalue`), which makes it easy to identify and exclude them.
  However, older bash versions do not print the prefixed `declare` statement, so the only way to
  identify them is by name, hence they must must be on the exclude list.

**Variable name constraints**

This runner currently does not support variable names that consist of other characters than
letters, digits and underscore.

Background: In `bash` version 4 the output of `declare -p` does not escape variable names.
Some people (looking at you, Windows) use variable names that contain characters which are special
to bash (like like `(` or `!`).
Those characters will result in statements like:

```sh
declare -- !C:
declare -x ProgramFiles(x86)
```

Executing these statements during import results in error, because bash will interpolate the special
characters.
