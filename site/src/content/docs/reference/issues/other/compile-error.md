---
title: CompileError
code: MIR1401
description: PHP rejects the code at compile time.
sidebar:
  hidden: true
  order: 8
---

The file parses, but PHP rejects it when compiling it: `php -l` prints `Fatal error` rather than `Parse error`. Examples are a non-static `__callStatic()`, a property in an interface without hooks, or a duplicate property.

The message ends with the parser's identifier for the error, e.g. `[magic-method-static]` or `[duplicate-property]`. The same identifier is the `code` field of the issue in `--format json`. Unlike the message wording, it stays the same between parser releases.

Unlike a syntax error, a compile error does not stop mir from analyzing the rest of the file.

## Example

```php
<?php
class A {
    public function __callStatic($name, $args) {} // must be static
}
```

## How to fix

Change the declaration as the message says. Run `php -l file.php` to see the same error from PHP.
