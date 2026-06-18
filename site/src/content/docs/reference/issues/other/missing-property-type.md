---
title: MissingPropertyType
code: MIR1104
description: "A class property has no declared type."
sidebar:
  hidden: true
  order: 1104
---

A class property has no declared type.

## Example

```php
<?php
class C { public $value; } // MissingPropertyType
```

## How to fix

Add a property type, e.g. `public int $value;`.
