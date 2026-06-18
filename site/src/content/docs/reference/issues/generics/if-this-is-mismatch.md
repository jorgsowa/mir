---
title: IfThisIsMismatch
code: MIR0902
description: "A method annotated `@if-this-is X<Y>` was called on a receiver whose type does not satisfy that constraint."
sidebar:
  hidden: true
  order: 902
---

A method annotated `@if-this-is X<Y>` was called on a receiver whose type does not satisfy that constraint.

## Example

```php
<?php
/** @if-this-is Collection<int> */ // called on Collection<string>
```

## How to fix

Call the method only when the receiver matches the constraint.
