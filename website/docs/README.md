---
sidebar_position: 1
---


import {customFields} from '@site/constants';


# What is Scrut?

Scrut is a CLI testing toolkit designed to rigorously test terminal programs. It is inspired by [Cram](https://github.com/brodie/cram) and focuses on providing a straightforward way to validate CLI behavior.

# Why though?

Testing command-line interfaces (CLIs) can be challenging. While unit tests are essential for verifying business logic, they often fall short in testing interactions with the environment, such as command line inputs, environment variables, and external APIs.

CLIs vary widely in complexity, from simple shell scripts automating repetitive tasks to large applications with multiple sub-commands. A robust testing framework should accommodate this diversity.

Scrut is a tool designed for functional, integration, end-to-end, and black-box testing of CLIs. It supports a flexible syntax to validate the behavior of CLIs, regardless of their complexity or the programming language used (e.g., Rust, Java, Ruby, C++, Typescript).

Scrut's key features include:

- **Low Threshold**: Easy to learn, with minimal entry barriers.
- **Low Effort**: Simplifies writing tests for even small CLI scripts.
- **Low Maintenance**: Ensures tests remain relevant and easy to update.
- **Readable**: Provides documentation value through clear test cases.

# How then?

The `scrut` command line application executes tests of CLIs that are persisted in Markdown (`.md`) or Cram (`.t`) files.

This is a markdown document that contains a test case:

````markdown title="my-test-document.md" showLineNumbers
# Scrut Tests are Markdown Documents

Write tests in code-blocks with the `scrut` language where you validate that
the output of a command matches your expectations.

## Version is printed

```scrut
$ acme --version
acme 1.0.0
```

There is plenty space to note down your intentions and whatever context you
want to provide to make this test maintainable.
````

Assuming the `acme` CLI is in your `PATH` then you can run the test with:

```bash title="Terminal"
$ scrut test my-test-document.md
🔎 Found 1 test document(s)

Result: 1 document(s) with 1 testcase(s): 1 succeeded, 0 failed and 0 skipped
```

That is assuming `acme --version` prints `acme 1.0.0`.


<FbInternalOnly>

## Getting Started

- [Open Source Tutorial](/docs/tutorial/)
- [Getting Started in Meta](/docs/fb/getting-started/)

</FbInternalOnly>

<OssOnly>

Head over to the [Getting Started Guide](/docs/getting-started/) to get learn how to write your own tests and everything else you need to know.

## Contribute

<ul>
    <li>
        <a href={ customFields.ossRepoUrl + "/blob/main/CONTRIBUTING.md" }>CONTRIBUTING.md</a>
    </li>
    <li>
        <a href={ customFields.ossRepoUrl + "/blob/main/CODE_OF_CONDUCT.md" }>CODE_OF_CONDUCT.md</a>
    </li>
</ul>

## License

<ul>
    <li>
        <a href={ customFields.ossRepoUrl + "/blob/main/LICENSE" }>LICENSE</a>
    </li>
</ul>

</OssOnly>
