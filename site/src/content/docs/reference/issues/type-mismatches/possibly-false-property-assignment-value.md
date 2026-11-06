---
title: PossiblyFalsePropertyAssignmentValue
code: MIR0233
description: "A value assigned to a property may be `false`, which the property's type doesn't accept."
sidebar:
  hidden: true
  order: 233
---

A value assigned to a property may be `false`, which the property's type doesn't accept.

## Example

```php
<?php
class Counter {
    public int $n = 0;

    public function set(int|false $value): void {
        $this->n = $value;
    }
}
```

## How to fix

Handle the `false` result (often a failure return) before assigning.
