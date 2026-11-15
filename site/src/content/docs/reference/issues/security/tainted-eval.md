---
title: TaintedEval
code: MIR0813
description: User-controlled input reaches `eval()` or code that will run.
sidebar:
  hidden: true
  order: 813
---

User-controlled input reaches code that will run, letting an attacker run arbitrary PHP: the code passed to `eval()`, or the content `file_put_contents()` writes to a path ending in `.php`, `.phtml`, `.phar` or `.pht`, the way cache and config writers turn into remote code execution. `var_export()` output is a valid PHP literal, so a value passed through it is safe here.

## Example

```php
<?php
eval($_GET['code']); // TaintedEval
```

## How to fix

Don't evaluate input. Parse it into data and act on that instead.
