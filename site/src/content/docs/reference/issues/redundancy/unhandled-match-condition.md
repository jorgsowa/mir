---
title: UnhandledMatchCondition
code: MIR0405
description: "A `match` has no arm for some possible subject value and no `default` arm."
sidebar:
  hidden: true
  order: 405
---

A `match` has no arm for some possible subject value and no `default` arm.

## Example

```php
<?php
$x = 1; match($x) { 0 => 'a' }; // UnhandledMatchCondition
```

## How to fix

Add the missing arm or a `default` arm.
