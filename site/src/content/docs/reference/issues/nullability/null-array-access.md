---
title: NullArrayAccess
description: Array access is performed on a value that is possibly null.
sidebar:
  order: 4
---

Array access is performed on a value that is possibly `null`.

## Example

```php
<?php
function getList(): ?array { return null; }

echo getList()[0]; // may be null
```

## How to fix

Guard with a null check before accessing array elements.
