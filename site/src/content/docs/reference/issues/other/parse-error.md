---
title: ParseError
description: A PHP file could not be parsed.
sidebar:
  order: 7
---

A PHP file could not be parsed due to a syntax error.

## Example

```php
<?php
function broken( { // missing parameter list
}
```

## How to fix

Fix the syntax error in the file. Run `php -l file.php` for a quick syntax check.
