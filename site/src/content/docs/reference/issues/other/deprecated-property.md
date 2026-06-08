---
title: DeprecatedProperty
code: MIR1005
description: Access to a `@deprecated` property.
sidebar:
  hidden: true
  order: 1005
---

A property annotated with `@deprecated` in its docblock is being read or written. Deprecated
properties are scheduled for removal and should not be used in new code.

## Example

```php
<?php
class User {
    /**
     * @deprecated Use $displayName instead.
     */
    public string $username = '';

    public string $displayName = '';
}

$user = new User();
echo $user->username; // deprecated property access
```

## How to fix

Switch to the replacement property or method indicated in the `@deprecated` message.
