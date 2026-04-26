---
title: DeprecatedClass
description: A class marked @deprecated is being instantiated.
sidebar:
  order: 2
---

A class marked `@deprecated` is being instantiated.

## Example

```php
<?php
/** @deprecated Use NewMailer instead */
class OldMailer {}

$m = new OldMailer(); // deprecated
```

## How to fix

Switch to the recommended replacement class.
