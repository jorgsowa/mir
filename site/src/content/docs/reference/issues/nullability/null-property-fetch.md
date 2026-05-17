---
title: NullPropertyFetch
code: MIR0101
description: A property is accessed on a value that is possibly null.
sidebar:
  hidden: true
  order: 2
---

A property is accessed on a value that is possibly `null`.

## Example

```php
<?php
function getUser(): ?User { return null; }

echo getUser()->name; // getUser() may return null
```

## How to fix

Add a null check before the access, or use the null-safe operator `?->`.

```php
<?php
echo getUser()?->name;
```
