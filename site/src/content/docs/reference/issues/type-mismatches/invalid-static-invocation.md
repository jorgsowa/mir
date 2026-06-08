---
title: InvalidStaticInvocation
code: MIR0215
description: Non-static method called with `::` syntax.
sidebar:
  hidden: true
  order: 215
---

A method is called using the static call syntax (`ClassName::method()`) but the method is not
declared `static`. Calling a non-static method statically is deprecated in PHP and will produce
a warning or error.

## Example

```php
<?php
class Formatter {
    public function format(string $value): string {
        return strtoupper($value);
    }
}

echo Formatter::format('hello'); // format() is not static
```

## How to fix

Either declare the method `static`, or instantiate the class and call the method on the instance:

```php
<?php
$formatter = new Formatter();
echo $formatter->format('hello');
```
