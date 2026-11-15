---
title: TaintedCommandArgument
code: MIR0824
description: "User input becomes an argument of a program run without a shell and can pass it an option."
sidebar:
  hidden: true
  order: 824
---

Passing the command as an array to `proc_open()` or Symfony's `new Process()` avoids the shell, so the input can't inject shell syntax. The program still parses its arguments, though, and a value such as `--upload-pack=touch /tmp/pwned` for `git clone` or `-oProxyCommand=...` for `ssh` runs code (CWE-88). Reported for a tainted array element after the program name, unless a `--` element comes before it, the element starts with a literal (`'--branch=' . $b`), or the value was checked not to start with `-`: `str_starts_with($x, '-')`, `$x[0] === '-'`, `strpos($x, '-') === 0` or `substr($x, 0, 1) === '-'` failing, or `str_starts_with($x, 'https://')` holding.

## Example

```php
<?php
proc_open(['git', 'clone', $_GET['repo']], $spec, $pipes); // TaintedCommandArgument
```

## How to fix

Put `--` before user-supplied arguments (`['git', 'clone', '--', $repo]`), or reject values that start with `-`.
