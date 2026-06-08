---
title: DeprecatedInterface
code: MIR1006
description: Implementing a `@deprecated` interface.
sidebar:
  hidden: true
  order: 1006
---

A class implements an interface that has been annotated with `@deprecated`. This interface is
scheduled for removal and should not be implemented in new code.

## Example

```php
<?php
/**
 * @deprecated Use NewContract instead.
 */
interface OldContract {
    public function process(): void;
}

class MyService implements OldContract { // implements deprecated interface
    public function process(): void {}
}
```

## How to fix

Implement the replacement interface indicated in the `@deprecated` message.
