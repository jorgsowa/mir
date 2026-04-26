---
title: UnusedMethod
description: A private method is never called within the class.
sidebar:
  order: 3
---

A private method is never called within the class.

## Example

```php
<?php
class Formatter {
    private function pad(string $s): string { // never called
        return str_pad($s, 10);
    }
}
```

## How to fix

Remove the method or call it from within the class.
