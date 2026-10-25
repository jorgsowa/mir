---
title: OptionalParameterBeforeRequired
code: MIR1012
description: An optional parameter is declared before a required one, which PHP deprecates from 8.0.
sidebar:
  hidden: true
  order: 10
---

A parameter with a default that comes before a required parameter can never be omitted, so PHP treats it as required and, from 8.0, emits a deprecation. The message follows the target version: 8.0 names each required parameter (`Required parameter $b follows optional parameter $a`), 8.1 and later each optional one (`Optional parameter $a declared before required parameter $b is implicitly treated as a required parameter`), and 8.4 prefixes the function name. A typed parameter with a `null` default is exempt the way PHP exempts it: always on 8.0, unless written `?T` on 8.1 and 8.2, and only when the type does not include `null` from 8.3.

## Example

```php
<?php
function pad(string $text = '', int $width): string
{
    return str_pad($text, $width);
}
```

## How to fix

Drop the default, or move the optional parameter after the required ones.
