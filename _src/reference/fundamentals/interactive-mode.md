# Interactive Mode

Interactive mode lets you test CLI programs that require live user interaction — reading input, responding to prompts, and verifying output as it appears on a pseudo-terminal (PTY). Instead of comparing captured stdout line-by-line, interactive tests drive a real terminal session through a sequence of **directives**.

Set `mode: interactive` via [inline configuration](/docs/reference/fundamentals/inline-configuration/#mode) to enable interactive mode for a test case.

## When to use interactive mode

Use interactive mode when your CLI:

- Prompts the user for input (e.g. `read`, `select`, `fzf`)
- Uses raw/cbreak terminal input (single-keypress UIs, arrow-key navigation)
- Produces output incrementally that depends on user interaction
- Requires testing the back-and-forth flow between user input and program response

For programs that simply print output to stdout and exit, use the default [`output` mode](/docs/reference/fundamentals/validation-modes/#output-default) instead.

## Directives

Interactive test bodies consist of **directives** — instructions that tell scrut what to do with the running PTY session. There are four directive types, each available in both long form and short form:

| Long form | Short | Purpose |
|-----------|-------|---------|
| `WAIT: <pattern>` | `@ <pattern>` | Wait for output matching a pattern |
| `WRITE: <text>` | `\| <text>` | Send text + newline to stdin |
| `SEND_KEYS: <tokens>` | `^ <tokens>` | Send raw keystrokes |
| `ASSERT: <pattern>` | `! <pattern>` | Assert current output matches pattern |

### `WAIT` / `@` — wait for output

Blocks until the terminal output contains text matching the given pattern, or until the timeout expires.

````markdown
```scrut {mode: interactive}
$ bash -c 'read -p "Name: " name && echo "Hello $name"'
WAIT: Name:
WRITE: Alice
WAIT: Hello Alice
```
````

An optional per-directive timeout overrides the test-case-level timeout:

```
WAIT {timeout: 5s}: Loading...
```

Or equivalently using a `%` modifier line before the directive:

```
% timeout: 5s
WAIT: Loading...
```

### `WRITE` / `|` — send text

Sends the given text followed by a newline to the PTY's stdin. This simulates the user typing a line and pressing Enter.

````markdown
```scrut {mode: interactive}
$ bash -c 'read -p "Flavor: " f && echo "You chose $f"'
@ Flavor:
| vanilla
@ You chose vanilla
```
````

### `SEND_KEYS` / `^` — send raw keystrokes

Sends one or more key tokens to the PTY without appending a newline. Each token is separated by a space and can be:

- **A single character**: `a`, `b`, `1`
- **A named key**: `ENTER`, `DOWN`, `UP`, `LEFT`, `RIGHT`, `TAB`, `ESCAPE`, `BACKSPACE`, `CTRL+C`
- **A hex byte**: `#0a` or `0x1b`

```
SEND_KEYS: a b ENTER
^ DOWN DOWN ENTER
```

#### Conditional key sending

`SEND_KEYS` supports optional conditions that check the current terminal output before sending keys:

```
SEND_KEYS {if_match: "Option A"}: ENTER
SEND_KEYS {unless_match: "already selected"}: SPACE
```

Or using `%` modifier lines:

```
% if_match: Option A
^ ENTER
```

### `ASSERT` / `!` — assert output

Checks the current terminal output immediately (non-blocking) against the given pattern. Unlike `WAIT`, `ASSERT` does not wait — it passes or fails based on what is already on screen.

````markdown
```scrut {mode: interactive}
$ echo "Hello World"
@ Hello
! Hello World
```
````

## Pattern matching

All pattern directives (`WAIT`, `ASSERT`) support scrut's standard pattern suffixes:

| Suffix | Match type |
|--------|-----------|
| *(none)* | Literal substring match (default) |
| `(regex)` | Regular expression match |
| `(glob)` | Glob pattern match (`*` = any sequence, `?` = any char) |

**Examples:**

```
WAIT: Version [0-9]+\.[0-9]+\.[0-9]+ loaded (regex)
WAIT: Config file: * (glob)
ASSERT: Total: [0-9]+ items (regex)
```

## Long form vs short form

Both forms are equivalent and can be mixed freely within a test case:

````markdown
Long form — explicit and readable:

```scrut {mode: interactive}
$ bash -c 'read -p "Name: " name && echo "Hello $name"'
WAIT: Name:
WRITE: test_user
WAIT: Hello test_user
```

Short form — compact:

```scrut {mode: interactive}
$ bash -c 'read -p "Name: " name && echo "Hello $name"'
@ Name:
| test_user
@ Hello test_user
```
````

## Inline configuration

Interactive mode is enabled via the `mode` configuration, either on the fence line or with `%` config lines:

````markdown
Fence-line config:

```scrut {mode: interactive}
$ my-interactive-command
@ prompt>
| input
```

Multiline `%` config:

```scrut
% mode: interactive
$ my-interactive-command
@ prompt>
| input
```
````

### Timeout

The test-case-level `timeout` sets the default timeout for all `WAIT` directives. Individual `WAIT` directives can override this with inline config:

````markdown
```scrut {mode: interactive, timeout: 30s}
$ slow-interactive-command
WAIT {timeout: 60s}: Eventually ready
WRITE: go
@ Done
```
````

### Multiline `%` modifier lines

Use `%` lines before a directive to set per-directive options. Modifier lines are buffered and applied to the next directive:

````markdown
```scrut
% mode: interactive
$ my-command
% timeout: 10s
@ Loaded
% unless_match: "already done"
^ ENTER
```
````

Supported modifiers:

| Modifier | Applies to | Effect |
|----------|-----------|--------|
| `% timeout: <duration>` | `WAIT` | Override wait timeout |
| `% if_match: <pattern>` | `SEND_KEYS` | Only send keys if pattern matches current output |
| `% unless_match: <pattern>` | `SEND_KEYS` | Only send keys if pattern does NOT match current output |
