---
title: QuotedShellEscape
code: MIR0825
description: "An `escapeshellarg()` result is wrapped in another pair of quotes in a shell command."
sidebar:
  hidden: true
  order: 825
---

`escapeshellarg()` returns its argument wrapped in single quotes. Putting that result between quotes again, `"ls '" . escapeshellarg($dir) . "'"` or `"grep \"$pattern\" file"`, makes the outer quotes end where the inner ones begin: inside single quotes the value runs unquoted, and inside double quotes `$(...)` still expands. Reported for `system()`, `exec()`, `shell_exec()`, `passthru()`, `popen()`, `proc_open()` and backticks, when the command text right before and after an `escapeshellarg()` call, or a variable assigned one, is the same quote character.

## Example

```php
<?php
$dir = escapeshellarg($_GET['dir']);
system("ls '$dir'"); // QuotedShellEscape
```

## How to fix

Drop the surrounding quotes: `system('ls ' . escapeshellarg($dir))`.
