---
title: PossiblyUndefinedVariable
description: A variable is only assigned in some branches and may be unset on other paths.
sidebar:
  order: 2
---

A variable is only assigned in some branches and may be unset on other paths.

## Example

```php
<?php
function label(bool $flag): string {
    if ($flag) {
        $text = 'yes';
    }
    return $text; // $text is not assigned when $flag is false
}
```

## How to fix

Assign a default before the branch, or handle every branch.

```php
<?php
function label(bool $flag): string {
    $text = 'no';
    if ($flag) {
        $text = 'yes';
    }
    return $text;
}
```
