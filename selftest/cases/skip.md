# Skipping the test from the inside

Scrut supports skipping of tests, that can be controlled from the "inside": If any shell expression of any test in a file ends in the exit code 80, then the whole file is skipped (ignored).

The use-case is e.g. test files that run only in special conditions (operating system (distribution)? environment / context? moon phase? ...)

This test file show-cases how that is done by having a test which would make the whole file fail, which is never triggered, because another test already exists in 80.

## Fail because of mismatching exit code

```scrut
$ echo OK
[1]
```

## Skip this file, and all failures, because of exit code 80

```scrut
$ exit 80
```

## Fail because of mismatching expectation

```scrut
$ echo OK
FAIL
```
