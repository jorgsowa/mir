---
title: PropertyPossiblyUninitialized
code: MIR1510
description: A property may not be initialized in all code paths before use.
sidebar:
  hidden: true
  order: 10
---

A property may not be initialized in all code paths before it is used, which can lead to undefined behavior.

## Example

```php
<?php
class User {
    public string $name;
    
    public function __construct(bool $condition) {
        if ($condition) {
            $this->name = 'Alice';
        }
        // $name may be uninitialized if condition is false
    }
    
    public function getName(): string {
        return $this->name; // Potentially uninitialized
    }
}
```

## How to fix

Ensure the property is initialized in all code paths:

```php
<?php
class User {
    public string $name;
    
    public function __construct(bool $condition) {
        if ($condition) {
            $this->name = 'Alice';
        } else {
            $this->name = 'Guest';
        }
    }
}
```
