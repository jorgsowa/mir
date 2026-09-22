---
title: ImmutablePropertyModification
code: MIR1705
description: An attempt to modify a property on an immutable object.
sidebar:
  hidden: true
  order: 5
---

An attempt to modify a property on an object that is marked as immutable (readonly class or readonly property).

## Example

```php
<?php
class User {
    public readonly string $name;
    
    public function __construct(string $name) {
        $this->name = $name;
    }
}

$user = new User('Alice');
$user->name = 'Bob'; // Error: cannot modify readonly property
```

## How to fix

Create a new instance with the updated value instead of modifying the existing one:

```php
<?php
class User {
    public readonly string $name;
    
    public function __construct(string $name) {
        $this->name = $name;
    }
    
    public function withName(string $name): self {
        return new self($name);
    }
}

$user = new User('Alice');
$updated = $user->withName('Bob');
```
