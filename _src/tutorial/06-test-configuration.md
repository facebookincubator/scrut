import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# Test Configuration

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

So far we have seen how to write test documents, how to execute them and how to work with the test environment. Once the rubber hits the road you will likely run into situations where you need to control the test execution behavior more closely. Scrut provides two ways of doing so:

## Inline Configuration

The preferred way is to persist the modification of the behavior in the test document itself. This way you can easily share the test document across test execution environments.

A common scenario is constraining the maximal execution time of individual test cases. For that purpose Scrut provides two timeout configuration options:
- *Per Test Document*,  constraining the cumulative execution time of all test cases in the test document.
- *Per Test Case*, constraining only the execution time of a single test case.

:::info

There are plenty more configuration options that are described in [Reference > Fundamentals > Inline Configuration](/docs/reference/fundamentals/inline-configuration/).

:::

### Per-Test-Document Configuration

The per-test-document timeout is defined in the test document's front matter ("a YAML snippet at the top of the Markdown document"). The following example leads to aborted test execution if the sum of executing all test cases exceeds 30 seconds.

````markdown title="tests/timeout.md" showLineNumbers {1-3}
---
total_timeout: 30s
---

# Test One

```scrut
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
* (glob+)
```

# Test Two

```scrut
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .some-other | valid-expression'
* (glob+)
```
````

### Per-Test-Case Configuration

Per-testcase configuration allows for much more granular control. This configuration must be provided as one-line YAML that is wrapped in curly brackets `{}`. The following example constraints the first test case execution to 10 seconds and the second one to twenty seconds.

````markdown title="tests/timeout.md" showLineNumbers {3,11}
# Test One

```scrut {timeout: 10s}
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
* (glob+)
```

# Test Two

```scrut {timeout: 20s}
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .some-other | valid-expression'
* (glob+)
```
````

### Default Per-Test-Case Timeout

If per-test-case configuration is shared within the document, then you can also overwrite default values in the per-document section of the configuration. The following sets a default timeout of 10 seconds for each test case, which is then later overwritten by the second test case to again 20 seconds:

````markdown title="tests/timeout.md" showLineNumbers {1-4,16}
---
defaults:
  timeout: 10s
---

# Test One

```scrut
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .commit.committer.date + "," + .commit.author.name'
* (glob+)
```

# Test Two

```scrut {timeout: 20s}
$ curl 'https://api.github.com/repos/jqlang/jq/commits?per_page=5' | \
    jq -r '.[] | .some-other | valid-expression'
* (glob+)
```
````

## Command Line Parameters

Many of the configuration options are mirrored by command line parameters. The per-document timeout described above could also be set using `--timeout-seconds=30` for the `scrut test` command. However, there is no equivalent for the `timeout` per-test-case configuration.

You can review all the available parameters by executing `scrut test --help` and respective `scrut update --help`.

:::warning

Using command line parameters **breaks the encapsulation of the test documents**. That means in order to replicate the test execution you need to know the command line arguments that were passed to `scrut test` - and their order!

:::
