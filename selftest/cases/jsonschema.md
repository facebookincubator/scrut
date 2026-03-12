# JSON Schema Validation

## Full Schema validates a JSON object

```scrut
% mode: jsonschema
$ echo '{"foo": "bar", "other": 123}'
---
"$schema": http://json-schema.org/draft-04/schema#
type: object
properties:
  foo:
    type: string
  other:
    type: integer
required:
- foo
- other
```

## Schema URL spec is optional

```scrut
% mode: jsonschema
$ echo '{"foo": "bar", "other": 123}'
---
type: object
properties:
  foo:
    type: string
  other:
    type: integer
required:
- foo
- other
```

## Unspecific attributes are valid

```scrut
% mode: jsonschema
$ echo '{"foo": "bar", "other": 123}'
---
type: object
properties:
  other:
    type: integer
required:
- other
```

Make them invalid with `additionalProperties: false`
