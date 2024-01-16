---
sidebar_position: 5
---


import GraphArchitecture from './_graph-architecture.md';
import GraphUseCase from './_graph-use-case.md';
import {customFields} from '@site/constants';


# Development

**!! Scrut is still actively developed. Major breaking changes are likely !!**

## Use Cases

Scrut is a command line program that solves the following use-cases for developers / owners of command line programs:

<GraphUseCase />

### Create Tests

Make it easy for owners to create tests for their CLIs. Accept arbitrary commands (or more complex shell expressions), execute them and create formatted tests from the resulting output.

Test Case generation is described by the <a href={ customFields.ossRepoUrl + '/blob/main/src/generators/generator.rs' }>`TestCaseGenerator`</a> trait and implemented in the respective format in the same <a href={ customFields.ossRepoUrl + '/blob/main/src/generators' }>folder</a>.

### Update Tests

Make it easy for owners to maintain the tests of their CLIs. Automate update of previously created test files when the expected output changes.

The generation of the update is described by the <a href={ customFields.ossRepoUrl + '/blob/main/src/generators/generator.rs' }>`UpdateGenerator`</a> trait and implemented in the respective format in the same <a href={ customFields.ossRepoUrl + '/blob/main/src/generators' }>folder</a>.

### Run Tests

Run previously persisted tests, so to prove that a CLI works within expectations. Owners can do this either manually, or automated from integration with their development tooling. The same tests should be run by automated continuous integration systems.

## Architecture

The architecture of Scrut is best explained by following the process flow of the primary use case: executing tests.

<GraphArchitecture />

### Phase: Parsing

Scrut tests are stored either in <a href={ customFields.ossRepoUrl + '/blob/main/src/parsers/markdown.rs' }>`Markdown`</a> or <a href={ customFields.ossRepoUrl + '/blob/main/src/parsers/cram.rs' }>`Cram`</a> files. Each file can contain multiple tests, which are called <a href={ customFields.ossRepoUrl + '/blob/main/src/testcase.rs' }>**Test Cases**</a> and which consist of:

- **Title** that explains to a human what this case is intended to prove
- **Shell Expression** is an arbitrary command or multiple chained commands, that result in a single result (exit code and output). For example: `date`, `date | awk '{print $1}'` and `date && date`
- <a href={ customFields.ossRepoUrl + '/blob/main/src/expectation.rs' }>**Expectations**</a> is a list of predictions in the form of rules that describe the output. For example: "_Output is exactly `Hello World`_" or "_Output start with `foo`_"
- **Exit Code** is the numeric code with which the shell expressions is expected to end (defaults to OK, aka `0`)

The Parsing phase extracts all testcases from the provided test file(s).

### Phase: Execution

The shell expression of the testcase needs to be executed in order to decide whether the output matches expectations. The <a href={ customFields.ossRepoUrl + '/blob/main/src/executors/executor.rs' }>`Executor`</a> is responsible to run a set of shell expressions. The <a href={ customFields.ossRepoUrl + '/blob/main/src/executors/stateful_executor.rs' }>`StatefulExecutor`</a> is currently used for executing Markdown files, and the <a href={ customFields.ossRepoUrl + '/blob/main/src/executors/bash_script_executor.rs' }>`BashScriptExecutor`</a> to execute Cram files.

The execution phase results in one <a href={ customFields.ossRepoUrl + '/blob/main/src/output.rs' }>`Output`</a> per testcase, that captures STDOUT, STDERR and the exit code.

### Phase: Validation

The output of execution for each testcase is checked against the expectations of the testcase. If the exit code mismatches, then the validation is immediately considered a failure and ends in an error.

If the exit code matches, then the output is compared line-by-line with the expectations by the <a href={ customFields.ossRepoUrl + '/blob/main/src/diff.rs' }>`DiffTool`</a>. If any comparison ends in the following states, then the whole validation is considered a failure:

- _Unmatched Expectation_: An expectation does not match any output line
- _Unexpected Output_: One or more lines of the output cannot be matched

### Phase: Presentation

Lastly the the outcome of the previous validation is renderer it into either a human readable diff-like text or a machine interpretable interchange format (JSON or YAML).
