import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# In-Depth Tutorial

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

:::info

This tutorial assumes you have [installed Scrut and learned the very basics of the command line](/docs/getting-started/). Building on that, you will learn all that you need to write meaningful tests for your own CLI(s).

:::

This guide is written with the following target audiences in mind:

- **CLI owners / contributors**, that care about the quality of a specific CLI and therefore want to
  - Validate the behavior of the CLI in the form of integration / end-to-end tests tests
  - Document the CLI behavior for themselves of future developers of the CLI
- **System administrators / operators**, that care about the CLI tools they work with and need to
  - Verify assumptions about their CLI tools
  - Document behavior of their CLI tools for themselves or future generations

## About File Structure

Scrut does not require any particular file structure. This tutorial is assuming that the files would be stored in a `tests` subdirectory together with the source code of the CLI that is being tested.

```bash
# Go to the directory where your CLI code lives
$ cd ~/Projects/MyCLI

# create a test folder
$ mkdir tests
```

:::tip

Although Scrut has no requirements towards file structure it is recommended that test relating files (like test fixtures, more about that later) are in the same directory structure as the test files themselves. This will make referencing them easier.

:::

## Using `jq` as an Example

Since Scrut is a CLI testing framework we need a CLI to test. Hence this tutorial will use the [powerful `jq` command-line JSON processor CLI](https://jqlang.org/) as an example CLI to write tests for. Make sure you have it [installed](https://jqlang.org/download/) if you want to follow along.
