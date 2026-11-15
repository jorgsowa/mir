---
title: TaintedMassAssignment
code: MIR0823
description: "User input chooses which property, variable, session key or named argument a write sets."
sidebar:
  hidden: true
  order: 823
---

User input chooses the name a write targets, so an attacker overwrites whatever they like: `$admin`, `$this->role` or a session flag. Reported for:

- `$obj->$name = ...` and `$$name = ...` with a tainted name;
- `$_SESSION[$key] = ...` with a tainted key;
- an array whose keys come from the request spread into a call, `f(...$_POST)`, `call_user_func_array($f, $_POST)` or `new $class(...$data)`: since PHP 8.0 string keys map to named parameters, so the attacker picks which parameters get set.

A name with a literal prefix (`$this->{'field_' . $k}`, `$_SESSION['cart_' . $id]`) can't reach other names and isn't reported, and neither is a value checked against an allowlist.

## Example

```php
<?php
foreach ($_POST as $key => $value) {
    $user->$key = $value; // TaintedMassAssignment
}
```

## How to fix

Copy the fields you expect by name, or check the name against an allowlist first: `in_array($key, ['name', 'email'], true)`.
