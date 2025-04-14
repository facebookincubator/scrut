# Working Directory

By default Scrut executes all tests in a dedicated directory *per [test document](/docs/reference/fundamentals/test-document/)*. This means *all [test cases](/docs/reference/fundamentals/test-case/) within one document are being executed in the same directory*. The directory is created within the system temporary directory. It will be removed (including all the files or directories that the tests may have created) after all tests in the file are executed - or if the execution of the file fails for any reason.

This means something like the following can be safely done and will be cleaned up by Scrut after the test finished (however it finishes):

````markdown title="test.md"
# Some test that creates a file

```scrut
$ date > file
```

The `file` lives in the current directory

```scrut
$ test -f "$(pwd)/file"
```
````

The directory within which tests are being executed can be explicitly set using the `--work-directory` parameter for the `test` and `update` commands. If that parameter is set then *all tests* from *all test files* are executed run within that directory, and the directory is *not removed* afterwards.

:::note

Consider also the environment variables `TESTDIR` and `TMPDIR` described in [Reference > Fundamentals > Environment Variables](/docs/reference/fundamentals/environment-variables/).

:::
