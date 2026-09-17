# JUnit Renderer

This test validates that the `junit` renderer emits a JUnit XML report that CI test result
collectors can consume.

## Run a document that has one passing and two failing test cases

```scrut
$ "$SCRUT_BIN" test --renderer junit --match-markdown "*.mdtest" "$TESTDIR/invalid.mdtest" > report.xml
[50]
```

## Report starts with an XML declaration and aggregated counts

```scrut
$ head -n 2 report.xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites tests="3" failures="2" errors="0" skipped="0" time="*"> (glob)
```

## All test cases of the document end up in a single suite

```scrut
$ grep -c "<testsuite " report.xml && grep -c "<testcase " report.xml
1
3
```

## Each test case is named after its title and points back at the source line

```scrut
$ grep -o 'name="This[^"]*"' report.xml
name="This test passes"
name="This test fails the output expectation"
name="This test fails the exit code expectation"
```

```scrut
$ grep -o 'line="[0-9]*"' report.xml
line="6"
line="13"
line="20"
```

## Every test case reports how long it took

```scrut
$ grep -c '<testcase [^>]*time="[0-9]*\.[0-9]*"' report.xml
3
```

## Failures carry the reason and the human readable diff

```scrut
$ grep -o '<failure type="[^"]*" message="[^"]*"' report.xml
<failure type="malformed_output" message="output does not match expectations"
<failure type="invalid_exit_code" message="unexpected exit code: expected 0, but got 3"
```

```scrut
$ grep -F -A 1 "1     | - expected" report.xml
1     | - expected
   1  | + actual</failure>
```

## Captured output is exposed as system-out

```scrut
$ grep -o "<system-out>[^<]*</system-out>" report.xml
<system-out>foo</system-out>
<system-out>actual</system-out>
```

## Every element that is opened is also closed again

```scrut
$ grep -c "</testcase>" report.xml && grep -c "</testsuite>" report.xml && grep -c "</testsuites>" report.xml
3
1
1
```
