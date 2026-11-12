===description===
`array + array` with a shape on the left and a generic array on the right keeps the right operand's value type for the keys it adds, and `[] + $array` is `$array` itself.
===file===
<?php
declare(strict_types=1);

final class Item
{
}

/** @return array<int, Item> */
function load(): array
{
    return [new Item()];
}

/** @return non-empty-array<int, Item> */
function loadNonEmpty(): array
{
    return [new Item()];
}

/** @return array<int, Item|null> */
function withRoot(): array
{
    $items = [0 => null] + load();
    /** @mir-check $items is array{0: null, ...<Item>} */
    return $items;
}

/** @return array<int, Item|null> */
function maybeRoot(bool $root): array
{
    $items = [];
    if ($root) {
        $items[0] = null;
    }
    $items = $items + load();
    /** @mir-check $items is array{0: null, ...<Item>}|array<int, Item> */
    return $items;
}

/** @return non-empty-array<int, Item> */
function nonEmpty(): array
{
    $items = [] + loadNonEmpty();
    /** @mir-check $items is non-empty-array<int, Item> */
    return $items;
}
