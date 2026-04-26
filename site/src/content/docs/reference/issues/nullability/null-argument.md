---
title: NullArgument
description: null is passed to a parameter that does not accept null.
sidebar:
  order: 1
---

`null` is passed to a parameter that does not accept `null`.

## Example

```php
<?php
function send(string $to): void {}

send(null); // string does not accept null
```

## How to fix

Pass a non-null value or widen the parameter type to `?string`.
