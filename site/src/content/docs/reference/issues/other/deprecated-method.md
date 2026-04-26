---
title: DeprecatedMethod
description: A method marked @deprecated is being called.
sidebar:
  order: 1
---

A method marked `@deprecated` is being called.

## Example

```php
<?php
class Api {
    /** @deprecated Use newMethod() instead */
    public function oldMethod(): void {}
}

$api = new Api();
$api->oldMethod(); // deprecated
```

## How to fix

Switch to the recommended replacement method.
