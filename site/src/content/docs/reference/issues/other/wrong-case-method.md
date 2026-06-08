---
title: WrongCaseMethod
code: MIR1010
description: Method name casing does not match its declaration.
sidebar:
  hidden: true
  order: 1010
---

A method is called with a casing that differs from its declaration. Although PHP method calls
are case-insensitive at runtime, consistent casing is required for clarity and tooling support.

## Example

```php
<?php
class Formatter {
    public function formatDate(\DateTimeInterface $date): string {
        return $date->format('Y-m-d');
    }
}

$f = new Formatter();
echo $f->FormatDate(new \DateTime()); // wrong casing: should be formatDate
```

## How to fix

Update the call site to use the exact casing from the method declaration.
