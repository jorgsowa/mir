---
title: UnusedProperty
description: A private property is never read within the class.
sidebar:
  order: 4
---

A private property is never read within the class.

## Example

```php
<?php
class Config {
    private string $debug = 'off'; // never read
}
```

## How to fix

Remove the property or read it in the class body.
