---
title: ReadonlyPropertyAssignment
description: A readonly property is assigned after the constructor.
sidebar:
  order: 6
---

A `readonly` property is assigned after the constructor.

## Example

```php
<?php
class User {
    public readonly string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }

    public function rename(string $name): void {
        $this->name = $name; // not allowed after construction
    }
}
```

## How to fix

Remove the post-construction assignment; `readonly` properties can only be set once during construction.
