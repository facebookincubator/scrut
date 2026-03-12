# Validation Modes

Scrut supports multiple **validation modes** that control how the output of a [shell expression](/docs/reference/fundamentals/shell-expression/) is verified against the expectations in a [test case](/docs/reference/fundamentals/test-case/). The mode is set via the [`mode` inline configuration](/docs/reference/fundamentals/inline-configuration/#mode).

## `output` (default)

The default mode. Scrut captures the command's stdout (or stderr / combined, depending on [`output_stream`](/docs/reference/fundamentals/inline-configuration/#output_stream)) and compares it line-by-line against the [output expectations](/docs/reference/fundamentals/output-expectations/) listed after the shell expression.

Each expectation line is matched using one of scrut's rule kinds — literal (default), `(regex)`, `(glob)`, or `(re)` — and the result is presented as a unified diff on failure.

**Example:**

````markdown
```scrut
$ echo "Hello World"
Hello World
```
````

Regex and glob expectations work as usual:

````markdown
```scrut
$ date +%Y-%m-%d
\d{4}-\d{2}-\d{2} (regex)
```
````

## `jsonschema`

JSON Schema mode validates the command's JSON output against an inline YAML schema instead of comparing individual output lines. This is useful for CLI tools that emit structured JSON — verifying shape, types, and required fields without fragile string matching.

### Syntax

Set `mode: jsonschema` via inline configuration. The expectation body starts with a `---` line followed by a YAML-formatted [JSON Schema](https://json-schema.org/):

````markdown
```scrut
% mode: jsonschema
$ echo '{"name": "scrut", "version": 1}'
---
type: object
properties:
  name:
    type: string
  version:
    type: integer
required:
  - name
  - version
```
````

Or equivalently with fence-line config:

````markdown
```scrut {mode: jsonschema}
$ echo '{"name": "scrut", "version": 1}'
---
type: object
properties:
  name:
    type: string
  version:
    type: integer
required:
  - name
  - version
```
````

### How it works

1. The YAML block after `---` is parsed as a JSON Schema.
2. The command's stdout is parsed as JSON.
3. The JSON output is validated against the schema.
4. On failure, scrut reports one of three error kinds:

| Error kind | Meaning |
|---|---|
| **InvalidSchema** | The YAML block is not a valid JSON Schema |
| **InvalidJson** | The command's output is not valid JSON |
| **ValidationErrors** | The JSON is valid but does not conform to the schema |

### Unspecified properties

By default, additional properties not listed in the schema are allowed. Use `additionalProperties: false` to reject them:

````markdown
```scrut {mode: jsonschema}
$ echo '{"name": "scrut", "extra": true}'
---
type: object
properties:
  name:
    type: string
required:
  - name
additionalProperties: false
```
````

In this example the test **fails** because `"extra"` is not declared in `properties`.

### Optional `$schema`

You may include a `$schema` URL in the YAML block, but it is not required:

```yaml
"$schema": http://json-schema.org/draft-04/schema#
type: object
```
