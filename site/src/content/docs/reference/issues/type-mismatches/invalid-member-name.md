---
title: InvalidMemberName
code: MIR0235
description: An object used as a dynamic property or method name.
sidebar:
  hidden: true
  order: 235
---

A dynamic member name (`$obj->{$name}` or `$obj->{$name}()`) is an object PHP cannot use as a
name. A property name is cast to string, so it must implement `Stringable`; otherwise PHP throws
`Error: Object of class X could not be converted to string`. A method name is never cast, so any
object throws `Error: Method name must be a string`.

## Example

```php
<?php
final class Name implements Stringable {
    public function __toString(): string { return 'id'; }
}

function read(object $target, Name $name): mixed {
    return $target->{$name}(); // method names must be strings
}
```

## How to fix

Cast the name to a string first:

```php
<?php
function read(object $target, Name $name): mixed {
    return $target->{(string) $name}();
}
```
