---
title: UnusedFunction
description: A function is defined but never called.
sidebar:
  order: 5
---

A function is defined but never called.

## Example

```php
<?php
function legacyHelper(): void { // never called
    // ...
}
```

## How to fix

Remove the function or add a call to it.
