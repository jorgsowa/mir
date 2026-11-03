---
title: TaintedFile
code: MIR0811
description: User-controlled input reaches a filesystem path.
sidebar:
  hidden: true
  order: 811
---

User-controlled input reaches the path argument of a filesystem function (`fopen()`, `file_get_contents()`, `file_put_contents()`, `unlink()`, `new SplFileObject()`, ...), letting an attacker read or write files outside the intended directory. Writing tainted *data* to a constant path is not reported.

## Example

```php
<?php
return file_get_contents($_GET['path']); // TaintedFile
```

## How to fix

Resolve the path with `realpath()` and reject it unless it is inside an allowed base directory, or reduce it to a file name with `basename()`.
