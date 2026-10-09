===description===
Untyped closure params of array_map/filter/reduce/walk/usort family take the array's element and key types.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Item {
    public function name(): string { return 'x'; }
}

/**
 * @param list<Item> $items
 * @param array<string, Item> $byKey
 */
function seeded(array $items, array $byKey): void {
    array_map(function ($i) {
        /** @mir-check $i is Item */
        return $i->name();
    }, $items);

    array_filter($items, fn($i) => $i->name() !== '');
    array_filter($byKey, function ($k) {
        /** @mir-check $k is string */
        return $k !== '';
    }, ARRAY_FILTER_USE_KEY);
    array_filter($byKey, function ($i, $k) {
        /** @mir-check $i is Item */
        /** @mir-check $k is string */
        return $i->name() === $k;
    }, ARRAY_FILTER_USE_BOTH);

    usort($items, function ($a, $b) {
        /** @mir-check $a is Item */
        /** @mir-check $b is Item */
        return $a->name() <=> $b->name();
    });
    uksort($byKey, function ($a, $b) {
        /** @mir-check $a is string */
        return strcmp($a, $b);
    });

    array_walk($byKey, function ($v, $k) {
        /** @mir-check $v is Item */
        /** @mir-check $k is string */
    });

    array_reduce($items, function ($carry, $i) {
        /** @mir-check $i is Item */
        return $carry . $i->name();
    }, '');

    array_map(function ($i, $k) {
        /** @mir-check $i is Item */
        /** @mir-check $k is int */
        return $k . $i->name();
    }, $items, array_keys($items));
}
