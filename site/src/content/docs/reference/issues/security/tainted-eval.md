---
title: TaintedEval
code: MIR0813
description: User-controlled input reaches `eval()`.
sidebar:
  hidden: true
  order: 813
---

User-controlled input reaches the code passed to `eval()`, letting an attacker run arbitrary PHP.

## Example

```php
<?php
eval($_GET['code']); // TaintedEval
```

## How to fix

Don't evaluate input. Parse it into data and act on that instead.
