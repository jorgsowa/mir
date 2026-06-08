---
title: DuplicateClass
code: MIR1602
description: Two classes with the same fully-qualified name exist in the codebase.
sidebar:
  hidden: true
  order: 1602
---

Two or more files define a class, interface, trait, or enum with the same fully-qualified name.
PHP will produce a fatal error if both definitions are loaded in the same request.

## Example

```php
<?php
// file: src/Models/User.php
namespace App\Models;
class User {}

// file: src/Legacy/User.php
namespace App\Models;
class User {} // duplicate of App\Models\User
```

## How to fix

Rename one of the classes, or move it to a different namespace to ensure each fully-qualified
name is unique across the project.
