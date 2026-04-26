---
title: MissingThrowsDocblock
description: A function throws an exception that is not declared in its @throws docblock.
sidebar:
  order: 5
---

A function throws an exception that is not declared in its `@throws` docblock.

## Example

```php
<?php
function open(string $path): resource {
    if (!file_exists($path)) {
        throw new \RuntimeException("Not found: $path"); // not declared in @throws
    }
    return fopen($path, 'r');
}
```

## How to fix

Add a `@throws \RuntimeException` annotation to the docblock.
