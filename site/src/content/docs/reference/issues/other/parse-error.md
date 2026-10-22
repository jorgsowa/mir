---
title: ParseError
code: MIR1400
description: A PHP file could not be parsed.
sidebar:
  hidden: true
  order: 7
---

A PHP file could not be parsed due to a syntax error, or uses syntax that the target PHP version does not have. Code that parses but that PHP rejects at compile time is reported as [CompileError](../compile-error/).

The message ends with the parser's identifier for the error, e.g. `[expected]`, `[version-too-low]` or `[invalid-assign-target]`. The same identifier is the `code` field of the issue in `--format json`. Unlike the message wording, it stays the same between parser releases.

## Example

```php
<?php
function broken( { // missing parameter list
}
```

## How to fix

Fix the syntax error in the file. Run `php -l file.php` for a quick syntax check.
