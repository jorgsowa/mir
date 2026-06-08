---
title: DeprecatedTrait
code: MIR1007
description: Using a `@deprecated` trait.
sidebar:
  hidden: true
  order: 1007
---

A trait annotated with `@deprecated` is being used via a `use` statement. Deprecated traits are
scheduled for removal and should not be used in new code.

## Example

```php
<?php
/**
 * @deprecated Use the new LoggingTrait instead.
 */
trait OldLoggingTrait {
    public function log(string $msg): void {}
}

class MyClass {
    use OldLoggingTrait; // uses deprecated trait
}
```

## How to fix

Replace the deprecated trait with its documented successor.
