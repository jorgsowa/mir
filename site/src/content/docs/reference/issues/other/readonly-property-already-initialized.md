---
title: ReadonlyPropertyAlreadyInitialized
code: MIR0601
description: A readonly property is assigned after it has already been initialized.
sidebar:
  hidden: true
  order: 1
---

A readonly property is being assigned a value after it has already been initialized. Readonly properties can only be assigned once, typically in the constructor.

## Example

```php
<?php
class User {
    public readonly string $name;
    
    public function __construct(string $name) {
        $this->name = $name;
    }
    
    public function rename(string $newName): void {
        $this->name = $newName; // Error: readonly property already initialized
    }
}
```

## How to fix

Remove the assignment or use a different design pattern:

```php
<?php
class User {
    public readonly string $name;
    
    public function __construct(string $name) {
        $this->name = $name;
    }
    
    // Instead of modifying, create a new instance
    public function withName(string $name): self {
        return new self($name);
    }
}
```
