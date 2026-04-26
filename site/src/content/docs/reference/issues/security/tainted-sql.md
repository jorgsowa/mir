---
title: TaintedSql
description: User-controlled input reaches a SQL sink without parameterization.
sidebar:
  order: 2
---

User-controlled input reaches a SQL sink without parameterization, creating a SQL injection risk.

## Example

```php
<?php
$id = $_GET['id'];
$db->query("SELECT * FROM users WHERE id = $id"); // injection risk
```

## How to fix

Use prepared statements with bound parameters.

```php
<?php
$stmt = $db->prepare('SELECT * FROM users WHERE id = ?');
$stmt->execute([$_GET['id']]);
```
