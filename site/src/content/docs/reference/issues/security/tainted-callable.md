---
title: TaintedCallable
code: MIR0810
description: User-controlled input chooses which function, method or class runs.
sidebar:
  hidden: true
  order: 810
---

User-controlled input is used as a callable or class name: `call_user_func()`, a callback argument, `$name()`, `$object->$name()` or `new $name()`. The attacker can then run arbitrary functions.

## Example

```php
<?php
$action = $_GET['action'];
$action(); // TaintedCallable
```

## How to fix

Map the input to an allowlist of callables, for example with `match`.
