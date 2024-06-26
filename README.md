# Scrut

Scrut is a testing toolkit for CLI applications. A tool to scrutinize terminal programs without fuzz. Heavily inspired by [Cram](https://github.com/brodie/cram).

# Why though?

Testing CLIs is complicated. Sure, you can (should!) write unit tests, as for any other software application, but they will (should!) only address your business logic and not test dependencies towards or interaction with the environment (command line input, environment variables, external APIs, etc).

What constitutes a CLI is not obvious or standardized. Some are small shell script, that consists of only a handful lines of code, which automate a thing that you are too lazy to do by hand more than once. Others are build upon tens of thousands of lines of code and feature multiple sub-commands that solve a range of related and complex problems. Often those large applications start out as small shell scripts and then evolve over time. You can call a program a CLI as long as it is intended to be executed from the shell.

A testing framework for CLIs should address all of that.

Scrut is a tool in the functional / integration / end-2-end / blackbox testing space. Tests are written in a flexible syntax that can be used to prove the behavior of any CLI - however complex and in whatever language (Rust, Java, Ruby, C++, Typescript, ...) they are written in.

Scrut aims to be

- **Low Threshold**, so that it's easy to learn and the entry barrier is very low (if you can execute commands on the terminal, you should be able to write a test)
- **Low Effort**, so that writing tests for even small CLI scripts is not (too) much overhead
- **Low Maintenance**, so that keeping tests around and up2date is not a painful chore
- **Readable**, so that tests can provide additional value to the reader as documentation (of the thing that is tested)

# How then?

The `scrut` command line application executes tests of CLIs that are persisted in Markdown (`.md`) or Cram (`.t`) files.

A very simple test looks like this:

````markdown
# Smoke test of the ACME CLI

This file contains a smoke test for the `acme` command line tool.
If it fails then things are seriously broken.

## Print version

```scrut
$ acme --version > /dev/null
```
````

To start using Scrut, see the [Tutorial](https://facebookincubator.github.io/scrut/docs/tutorial/) page for a walkthrough of Scrut use from start to end.

## Contribute

- [Contributing](CONTRIBUTING.md)
- [Code of Conducat](CODE_OF_CONDUCT.md)

## License

- [LICENSE](LICENSE)
