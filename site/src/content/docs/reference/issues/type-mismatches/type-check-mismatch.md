---
title: TypeCheckMismatch
code: MIR0212
description: A `@mir-check` assertion in a test fixture does not match the inferred type.
sidebar:
  hidden: true
  order: 212
---

This is an **internal diagnostic** used in mir's own test fixtures. It is emitted when a
`@mir-check` assertion does not match the type mir inferred for the annotated expression.

`@mir-check` annotations are only meaningful inside `.phpt` fixture files under
`crates/mir-analyzer/tests/fixtures/`. They are not intended for use in production code.

## Example

```php
<?php
$x = 42;
/** @mir-check $x is string */ // TypeCheckMismatch: inferred type is int, not string
echo $x;
```

## How to fix

Correct the `@mir-check` assertion to match the actual inferred type, or fix the code so that
the inferred type matches the intended assertion. This issue will not appear in normal project
analysis.
