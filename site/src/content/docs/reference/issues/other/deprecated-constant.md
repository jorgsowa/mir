---
title: DeprecatedConstant
code: MIR1008
description: Reference to a `@deprecated` constant.
sidebar:
  hidden: true
  order: 1008
---

A constant annotated with `@deprecated` is being referenced. Deprecated constants are scheduled
for removal and should not be used in new code.

## Example

```php
<?php
class Status {
    /**
     * @deprecated Use Status::ACTIVE instead.
     */
    public const ENABLED = 1;

    public const ACTIVE = 1;
}

echo Status::ENABLED; // deprecated constant
```

## How to fix

Use the replacement constant indicated in the `@deprecated` message.
