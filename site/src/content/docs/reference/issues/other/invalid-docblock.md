---
title: InvalidDocblock
description: A docblock contains a malformed or unrecognised annotation.
sidebar:
  order: 8
---

A docblock contains a malformed or unrecognised annotation.

## Example

```php
<?php
/**
 * @return int|
 */
function getId(): int { return 1; } // trailing | is invalid type syntax
```

## How to fix

Fix the annotation syntax. Refer to [Docblock Annotations](/mir/reference/docblock/) for supported forms.
