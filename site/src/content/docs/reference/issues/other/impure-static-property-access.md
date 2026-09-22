---
title: ImpureStaticPropertyAccess
code: MIR1708
description: An impure function reads a static property, depending on external state.
sidebar:
  hidden: true
  order: 8
---

An impure function reads a static property, making it dependent on external state that can change unexpectedly.

## Example

```php
<?php
class Config {
    public static $debug = false;
}

function isDevelopment(): bool {
    return Config::$debug; // Depends on external static state
}
```

## How to fix

Pass dependencies explicitly or use dependency injection:

```php
<?php
class Config {
    private bool $debug;
    
    public function __construct(bool $debug) {
        $this->debug = $debug;
    }
    
    public function isDevelopment(): bool {
        return $this->debug;
    }
}
```
