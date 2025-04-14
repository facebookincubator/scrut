import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Output Expectations

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

Smoke tests are useful for identifying if a program is broken, but they don't confirm correct functionality. When running commands manually in the terminal, the initial check for correct operation is through their output: does it match expectations, or are there error messages?

This is what **output expectations** are all about.

## Output Expectation Types

The simplest variant of an *output expectation* was already demonstrated previously when the test for the `jq --version` command was created:

````markdown {3}
```scrut
$ jq --version
jq-1.7.1
```
````

The line that reads `jq-1.7.1` is what Scrut calls a *equal* output expectation. It could also have been written like this:

````markdown {3}
```scrut
$ jq --version
jq-1.7.1 (equal)
```
````

The suffix ` (equal)` here tells Scrut that the output is expected exactly as written before. There are other types, for example:

**Glob: Match all**

````markdown {3}
```scrut
$ jq --version
jq-* (glob)
```
````

The `*` wildcard in `jq-*` matches anything. Scrut would accept any string that starts with `jq-`.

**Regex: Match precisely**

````markdown {3}
```scrut
$ jq --version
jq-1\.\d+\.\d+ (regex)
```
````

The `1\.\d+\.\d+` regular expression matches any version number that starts with a one and is followed by two numbers, separated by a dot.

:::info

Learn more about output expectations in the [Reference > Fundamentals > Output Expectations](/docs/reference/fundamentals/output-expectations/) later.

:::

## Practical Example

Let's take a look at a practical example. Using `jq` some JSON input data is required. Following the same example as provided in the [`jq` tutorial itself](https://jqlang.org/tutorial/): Let's go with the Github API.

```bash title="Terminal"
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5'
[
  {
    "sha": "947fcbbb1fedbdd6021ef3f93782a500e32d5dcd",
    "node_id": "C_kwDOAE3WVdoAKDk0N2ZjYmJiMWZlZGJkZDYwMjFlZjNmOTM3ODJhNTAwZTMyZDVkY2Q",
    "commit": {
      "author": {
        "name": "dependabot[bot]",
        "email": "49699333+dependabot[bot]@users.noreply.github.com",
        "date": "2025-03-28T00:57:51Z"
      },
      "committer": {
        "name": "GitHub",
        "email": "noreply@github.com",
        "date": "2025-03-28T00:57:51Z"
      },
      "message": "--%<--",
      "tree": {
        "sha": "8b30ae1036b74c4acf02c674f75db8f1ce014aa4",
        "url": "https://api.github.com/repos/jqlang/jq/git/trees/8b30ae1036b74c4acf02c674f75db8f1ce014aa4"
      },
      "url": "https://api.github.com/repos/jqlang/jq/git/commits/947fcbbb1fedbdd6021ef3f93782a500e32d5dcd",
      "comment_count": 0,
      "verification": {
        "verified": true,
        "reason": "valid",
        "signature": "--%<--",
        "payload": "--%<--",
        "verified_at": "2025-03-28T00:57:55Z"
      }
    },
    "url": "https://api.github.com/repos/jqlang/jq/commits/947fcbbb1fedbdd6021ef3f93782a500e32d5dcd",
    "html_url": "https://github.com/jqlang/jq/commit/947fcbbb1fedbdd6021ef3f93782a500e32d5dcd",
    "comments_url": "https://api.github.com/repos/jqlang/jq/commits/947fcbbb1fedbdd6021ef3f93782a500e32d5dcd/comments",
    "author": {
--%<--
```

That is a lot of output. Let's use `jq` to boil that down into something more manageable. Say, as CSV with the first column the commit date and the second column the author's name:

```bash title="Terminal"
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
2025-03-28T00:57:51Z,dependabot[bot]
2025-03-28T00:56:51Z,dependabot[bot]
2025-03-28T00:55:39Z,dependabot[bot]
2025-03-27T23:43:06Z,itchyny
2025-03-27T23:42:44Z,itchyny
```

:::note

The output you will see when executing the above `curl` command will contain more lines than are shown above:

```bash title="Terminal" {3-5}
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100 28935  100 28935    0     0   250k      0 --:--:-- --:--:-- --:--:--  250k
2025-03-28T00:57:51Z,dependabot[bot]
2025-03-28T00:56:51Z,dependabot[bot]
2025-03-28T00:55:39Z,dependabot[bot]
2025-03-27T23:43:06Z,itchyny
2025-03-27T23:42:44Z,itchyny
```

The first three lines above that `curl` prints are written to STDERR. Only the actual result content (i.e. the web request body) is printed to STDOUT and piped to `jq` which transforms them into five lines that are finally printed on STDOUT.

**Scrut only considers STDOUT** by default. More about how to change this behavior [here](/docs/reference/behavior/stdout-and-stderr/).

:::

To go from here to a test either use `scrut create` with the above command, or open a new file and add the commandline and output yourself:

````markdown title="tests/expectations.md"
# Output Expectations

```scrut
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
>   jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
2025-03-28T00:57:51Z,dependabot[bot]
2025-03-28T00:56:51Z,dependabot[bot]
2025-03-28T00:55:39Z,dependabot[bot]
2025-03-27T23:43:06Z,itchyny
2025-03-27T23:42:44Z,itchyny
```
````

:::note

Shell expressions that span multiple lines need to be prefixed with a `>`, like so:

````markdown
```scrut
$ line 1
> line 2
> line N
```
````

The whole expression will then be piped into a bash process and executed. If you do not concatenate the lines with something `&&` or explicitly `set -e`, then the exit code will be from the last executed line.

````markdown
```scrut
$ line 1 && \
> line 2 && \
> line N
```
````

:::

### Generalize Output Expectation

Running `scrut test tests/expectations.md` right after creating the file should succeed. *Should*, because the output is not stable. It is not guaranteed to be the same tomorrow, or even in a few minutes. To make it more stable the test can be changed:
- from *with the JSON input from the github API this exact output is expected*
- to *with the JSON input from the github API 5 lines separated by a comma are expected*

````markdown {4-8}
```scrut
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
>   jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
*,* (glob)
*,* (glob)
*,* (glob)
*,* (glob)
*,* (glob)
```
````

Obviously this test lost precision compared to the previous variant, but on the plus side: it won't break as easy, it is still meaningful and it could break if, say, the `jq` concatenate operator `+` malfunctions. This could be made more precise using `20*T*Z,* (glob)` to account for the date string, or even use matching `regex` rules.

### Quantifiers for Expectations

The above test could be generalized further. While it probably would not make sense for this case the following would work as well:

````markdown {4}
```scrut
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
>   jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
*,* (glob+)
```
````

Note the `+` behind the word `glob`. This is a **quantifier**. Quantifiers can be used with any output expectation. They make sense when a hard to predict amount of predictable formatted output needs to be accounted for.

:::note

Scrut currently understands three quantifiers:
- `?`: Zero or one
- `*`: Any amount, including zero
- `+`: Any amount, but at least one

More detail in [Reference > Fundamentals > Output Expectations > Quantifiers](/docs/reference/fundamentals/output-expectations/#quantifiers).

:::
