---
title: UndefinedVariable
code: MIR0002
description: A variable is used before it has been assigned.
sidebar:
  hidden: true
  order: 1
---

A variable is used before it has been assigned.

## Example

```php
<?php
function greet(): string {
    return $message; // $message was never assigned
}
```

## How to fix

Assign the variable before reading it, or initialise it to a default value.

```php
<?php
function greet(): string {
    $message = 'hello';
    return $message;
}
```
