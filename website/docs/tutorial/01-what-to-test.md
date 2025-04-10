import OssTutorialNote from '../fb/components/_oss-tutorial-note.md';

# What to test?

<FbInternalOnly><OssTutorialNote /></FbInternalOnly>

CLIs are as diverse as the tasks they are designed to perform. The tests that validate an individual CLI must be tailored to the specific features and use-cases. On a very high level Scrut is designed to support the following types of tests:

- **Smoke Testing** - Validate that the CLI is fundamentally executable.
- **Functional Testing** - Validate that the CLI performs its intended functions.
- **Integration / End-to-End Testing** - Validate that the CLI works correctly when integrated with other systems.

So where do you start? What is good first test to write in any case?

All CLI share at least one trait: they all are have an interface to execute on the command line. So "being executable from the command line" is the testable trait. The type of test that addresses that, and therefore the aforementioned **Smoke test** is the best place to start.

:::info

A smoke test, in very short, is a very basic test that answers the question: *If I switch it on, do I see smoke rising?* Where *it* refers to an arbitrary electric device.

:::

For CLIs a smoke test means: *If I execute the CLI in the most basic way does it return a `0` exit code?*

What is a "basic way"? Think about `mycli --version` or `mycli --help`. Both parameters should exist for a well written CLI and executing either should *not run any significant business logic*. This is what makes them great candidates for a smoke test.

The value of testing such basic functionality of your CLI is that if the smoke test fails, then you know your CLI is fundamentally broken.

For the scope of this tutorial: Wether `jq --version` can be executed successfully is the first test we will write.
