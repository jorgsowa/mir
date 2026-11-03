---
title: TaintedInput
code: MIR0800
description: Tainted input reaches a `@taint-sink` parameter of a kind with no dedicated issue.
sidebar:
  hidden: true
  order: 800
---

Tainted input reaches a parameter annotated `@taint-sink <kind>` where `<kind>`
has no dedicated issue. The `sink` field names that kind. Built-in sinks and
the named kinds (`html`, `sql`, `shell`, `file`, `include`, `eval`,
`unserialize`, ...) report their own issue, such as
[`TaintedFile`](/reference/issues/security/tainted-file/).

## Example

```php
<?php
/** @taint-sink graphql $query */
function runQuery(string $query): void {}

runQuery($_GET['q']); // TaintedInput: Tainted input reaching sink 'graphql'
```

## How to fix

Validate or escape the value before it reaches the sink. If the value is
already trusted, suppress with `@mir-suppress TaintedInput`.
