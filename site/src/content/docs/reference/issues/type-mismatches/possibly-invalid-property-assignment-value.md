---
title: PossiblyInvalidPropertyAssignmentValue
code: MIR0234
description: "Only some types of a value assigned to a property match the property's type."
sidebar:
  hidden: true
  order: 234
---

Only some types of a value assigned to a property match the property's type.

## Example

```php
<?php
class Counter {
    public int $n = 0;

    public function set(int|string $value): void {
        $this->n = $value;
    }
}
```

## How to fix

Narrow the value to the property's type before assigning, or widen the property type.
