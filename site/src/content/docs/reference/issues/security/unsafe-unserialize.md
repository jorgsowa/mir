---
title: UnsafeUnserialize
code: MIR0820
description: "`unserialize()` of data that isn't a literal, without an `allowed_classes` restriction."
sidebar:
  hidden: true
  order: 820
---

Opt-in: enable it with an `<issueHandlers>` entry such as `<UnsafeUnserialize errorLevel="warning"/>`. Unlike `TaintedUnserialize`, it doesn't need proof that the data comes from the request, so it also catches second-order cases where the payload was stored in a database or cache an attacker can write to. Data already reported as `TaintedUnserialize` isn't reported again.

## Example

```php
<?php
$cart = unserialize($cache->get('cart')); // UnsafeUnserialize
```

## How to fix

Pass `['allowed_classes' => false]` or a list of expected classes, or store the data as JSON.
