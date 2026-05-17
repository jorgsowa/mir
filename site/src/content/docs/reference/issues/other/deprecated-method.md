---
title: DeprecatedMethod
code: MIR1002
description: A method marked @deprecated is being called.
sidebar:
  hidden: true
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
