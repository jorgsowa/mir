---
title: NullMethodCall
description: A method is called on a value that is possibly null.
sidebar:
  order: 3
---

A method is called on a value that is possibly `null`.

## Example

```php
<?php
function find(): ?Item { return null; }

find()->process(); // may be null
```

## How to fix

Guard with a null check or use the null-safe operator `?->`.

```php
<?php
find()?->process();
```
