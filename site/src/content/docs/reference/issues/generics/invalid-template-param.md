---
title: InvalidTemplateParam
description: A template argument violates the declared bounds of the type parameter.
sidebar:
  order: 1
---

A template argument violates the declared bounds of the type parameter.

## Example

```php
<?php
/**
 * @template T of Countable
 * @param T $collection
 */
function count_items($collection): int {
    return count($collection);
}

count_items('hello'); // string does not satisfy Countable bound
```

## How to fix

Pass a value that satisfies the template bound, or loosen the bound if appropriate.
