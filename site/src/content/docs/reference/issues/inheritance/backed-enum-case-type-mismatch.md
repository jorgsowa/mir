---
title: BackedEnumCaseTypeMismatch
code: MIR0713
description: A backed enum case has a value of the wrong type.
sidebar:
  hidden: true
  order: 13
---

A backed enum case has a value that doesn't match the enum's backing type.

## Example

```php
<?php
enum Status: int {
    case Draft = 'draft'; // Error: string value for int-backed enum
    case Published = 1;
}
```

## How to fix

Use values that match the enum's backing type:

```php
<?php
enum Status: int {
    case Draft = 0;
    case Published = 1;
}

// Or use string-backed enum
enum Status: string {
    case Draft = 'draft';
    case Published = 'published';
}
```
