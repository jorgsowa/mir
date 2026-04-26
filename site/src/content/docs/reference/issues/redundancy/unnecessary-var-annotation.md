---
title: UnnecessaryVarAnnotation
description: A @var annotation matches the type that mir already infers.
sidebar:
  order: 3
---

A `@var` annotation matches the type that mir already infers.

## Example

```php
<?php
/** @var int $count */
$count = count($items); // count() already returns int
```

## How to fix

Remove the `@var` annotation — it provides no additional information.
