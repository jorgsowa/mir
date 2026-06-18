---
title: PossiblyInvalidArrayAccess
code: MIR0227
description: "Array access on a value that is only sometimes offset-accessible."
sidebar:
  hidden: true
  order: 227
---

Array access on a value that is only sometimes offset-accessible.

## Example

```php
<?php
/** @param array|int $x */
function f($x){ return $x[0]; }
```

## How to fix

Narrow the type to an array (or `ArrayAccess`) before indexing.
