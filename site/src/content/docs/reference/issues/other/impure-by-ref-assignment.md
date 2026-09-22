---
title: ImpureByRefAssignment
code: MIR1707
description: An impure function modifies a parameter passed by reference, causing side effects.
sidebar:
  hidden: true
  order: 7
---

An impure function modifies a parameter that was passed by reference, which introduces side effects that can make code harder to reason about and test.

## Example

```php
<?php
function updateConfig(array &$config, string $key, mixed $value): void {
    $config[$key] = $value; // Modifies caller's array by reference
}
```

## How to fix

Return a new value instead of modifying by reference, or document the side effect clearly:

```php
<?php
function updateConfig(array $config, string $key, mixed $value): array {
    $config[$key] = $value;
    return $config;
}

// Usage
$config = updateConfig($config, 'key', 'value');
```
