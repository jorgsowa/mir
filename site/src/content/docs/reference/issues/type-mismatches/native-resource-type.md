---
title: NativeResourceType
code: MIR0236
description: "`resource` used as a native parameter, return or property type."
sidebar:
  hidden: true
  order: 236
---

PHP has no native `resource` type. In a signature, `resource` is read as a class name in the
current namespace, so passing an actual resource handle throws
`TypeError: f(): Argument #1 ($value) must be of type resource, resource given`. Only a PHPDoc
`resource` describes a resource handle.

## Example

```php
<?php
function close(resource $handle): void {
    fclose($handle);
}
```

## How to fix

Move the type to the docblock:

```php
<?php
/** @param resource $handle */
function close($handle): void {
    fclose($handle);
}
```
