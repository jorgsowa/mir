---
title: PossiblyNullPropertyAssignmentValue
code: MIR0232
description: "A value assigned to a property may be `null`, which the property's type doesn't accept."
sidebar:
  hidden: true
  order: 232
---

A value assigned to a property may be `null`, which the property's type doesn't accept.

## Example

```php
<?php
class Counter {
    public int $n = 0;

    public function set(?int $value): void {
        $this->n = $value;
    }
}
```

## How to fix

Check for `null` before assigning, or make the property nullable.
