---
title: TaintedEval
code: MIR0813
description: User-controlled input reaches `eval()` or code that will run.
sidebar:
  hidden: true
  order: 813
---

User-controlled input reaches code that will run, letting an attacker run arbitrary PHP: the code passed to `eval()`, or the content `file_put_contents()` writes to a path ending in `.php`, `.phtml`, `.phar` or `.pht`, the way cache and config writers turn into remote code execution. Template source compiled into code counts too: Twig's `Environment::createTemplate()`, Blade's `Blade::render()` and `BladeCompiler::render()`, and a Smarty `fetch()`, `display()` or `createTemplate()` of a `string:` or `eval:` resource (server-side template injection). `var_export()` output is a valid PHP literal, and `rawurlencode()` and `base64_encode()` output can't form PHP syntax, so values passed through them are safe here.

## Example

```php
<?php
eval($_GET['code']); // TaintedEval
```

## How to fix

Don't evaluate input. Parse it into data and act on that instead.
