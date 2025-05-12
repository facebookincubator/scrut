# Shell Expressions

At the core of any Scrut [test case](/docs/reference/fundamentals/test-case/) is a command line that is being tested. It can be a single command, a sequence of commands spanning multiple lines or anything else could possibly be written or pasted on the command line and executed. This command line is called a *shell expression*.

All of the following are valid shell expressions:

```bash title="Terminal"
$ date
$ echo Hello World
$ date && echo Hello World
$ function foo() { echo Hello World; }; foo
$ cat foo | grep bar | wc -l
$ function foo() { echo Hello World; }; \
  foo | \
  grep Hello | \
  wc -l
```

The rule of thumb is: If you can paste and excecute it in the shell, then it is a valid shell expression.

## Constraints

For the sake of understanding assume that each shell expression is written to a file and this file is then executed with `bash`. Like so:

```bash title="Terminal"
$ echo 'echo My shell expression' > shell-expression.sh
$ bash shell-expression.sh
My shell expression
```

:::note

Learn about the how execution works in [Reference > Behavior > Execution Model](/docs/reference/behavior/execution-model/).

:::

This behavior implies some limits / constraints on what you can expect from the result:

### Returned Exit Code

Consider the following [shell expression](/docs/reference/fundamentals/shell-expression/):

```bash
$ false ; true
```

This executes the command `false` and then executes the command `true`. They are both separated by a `;`, which makes them individual commands from the `bash` perspective. If you would have a bash script file with these contents and would execute it, then you would see the exit code `0`:

```bash title="Terminal"
$ echo 'false ; true' > shell-expression.sh
$ bash shell-expression.sh
$ echo $?
0
```

That means the exit code of `false` (which is `1`) is not surfaced, because the shell script itself continues to the next command (`true`). The returned exit code is simply the exit code of the last command in the shell script.

If this behavior is not desired (it may be), then you could use the `&&` operator instead of `;`:

```bash title="Terminal"
$ echo 'false && true' > shell-expression.sh
$ bash shell-expression.sh
$ echo $?
1
```

Alternatively, as these are bash scripts, you can also use the `set -e` directive to make the shell script exit on the first non-zero exit code:

```bash title="Terminal"
$ echo 'set -e ; false ; true' > shell-expression.sh
$ bash shell-expression.sh
$ echo $?
1
```

:::warning

Due to the different [execution model of Cram](/docs/reference/behavior/execution-model/) using `set -e` will terminate not only the [test case](/docs/reference/fundamentals/test-case/), but all [test cases](/docs/reference/fundamentals/test-case/) in the same [test document](/docs/reference/fundamentals/test-document/). Do not use it.

:::

### Detached Processes

When Scrut runs a shell expression it will wait for the execution to finish, so that it can gather the exit code and the output and validate it as defined by the [test case](/docs/reference/fundamentals/test-case/).

However, if the shell expression detaches from the shell, or spawns processes that are detached (or both) then Scrut will not wait for them. **Scrut will not manage their lifetime at all.**

:::tip

If you need to test a server/client scenario, where first a server must be started and before the CLI [test cases](/docs/reference/fundamentals/test-case/) can execute then have a look at the [`detached`/`wait` configuration directives](/docs/reference/fundamentals/inline-configuration/#wait-configuration).

Here the `detached_kill_signal` can be specified to send a user-definedable signal to the detached process to terminate it. **Note that Scrut will only send the signal, it is up to the process to handle it correctly.**

:::
