---
title: ParadoxicalCondition
code: MIR0404
description: Condition that directly contradicts itself (always false).
sidebar:
  hidden: true
  order: 404
---

A condition is constructed such that it can never be true — it contradicts itself. This is
distinct from `RedundantCondition` (which is always true): a paradoxical condition is always
false, so the controlled branch is dead code.

## Example

```php
<?php
function check(int $x): bool {
    return $x > 10 && $x < 5; // can never be true simultaneously
}
```

## How to fix

Review the condition logic. If the branch should sometimes execute, correct the operator or
bounds. If the branch is intentionally unreachable, remove it.
