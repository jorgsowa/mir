---
title: InternalMethod
description: A method marked @internal is called from outside its package.
sidebar:
  order: 3
---

A method marked `@internal` is called from outside its package.

## Example

```php
<?php
namespace Acme\Core;

class Registry {
    /** @internal */
    public function rebuild(): void {}
}

// In a different namespace:
(new \Acme\Core\Registry())->rebuild(); // @internal method accessed externally
```

## How to fix

Use only the public API; do not rely on internal implementation details.
