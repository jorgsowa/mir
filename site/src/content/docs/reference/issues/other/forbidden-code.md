---
title: ForbiddenCode
code: MIR1301
description: "Use of a forbidden construct such as `var_dump`, `shell_exec`, or the backtick operator."
sidebar:
  hidden: true
  order: 1301
---

Use of a forbidden construct such as `var_dump`, `shell_exec`, or the backtick operator.

## Example

```php
<?php
var_dump($x); // ForbiddenCode
```

## How to fix

Remove the debugging or forbidden call.
