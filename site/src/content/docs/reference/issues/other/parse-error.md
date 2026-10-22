---
title: ParseError
code: MIR1400
description: A PHP file could not be parsed.
sidebar:
  hidden: true
  order: 7
---

A PHP file could not be parsed due to a syntax error, or the parser found code PHP rejects at compile time.

The message ends with the parser's identifier for the error, e.g. `[expected]`, `[version-too-low]` or `[duplicate-property]`. The same identifier is the `code` field of the issue in `--format json`. Unlike the message wording, it stays the same between parser releases.

## Example

```php
<?php
function broken( { // missing parameter list
}
```

## How to fix

Fix the syntax error in the file. Run `php -l file.php` for a quick syntax check.
