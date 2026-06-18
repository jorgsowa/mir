---
title: UnsupportedReferenceUsage
code: MIR1506
description: "A PHP reference assignment is used in a form mir cannot model precisely (e.g. `$b = &$arr[$x]`)."
sidebar:
  hidden: true
  order: 1506
---

A PHP reference assignment is used in a form mir cannot model precisely (e.g. `$b = &$arr[$x]`).

## Example

```php
<?php
$b = &$arr[$x]; // UnsupportedReferenceUsage
```

## How to fix

Avoid the reference, or restructure to a plain assignment.
