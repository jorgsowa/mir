===description===
FP: `foreach` over a value whose *static* type is directly one of the
built-in iteration interfaces used generically — `@param Iterator<TKey,
TValue> $x`, `IteratorAggregate<TKey, TValue>`, or `Traversable<TKey,
TValue>` — rather than a concrete class implementing one of them, still
produced `mixed`/`mixed`. There's no `current()`/`getIterator()` to chase
in that case; the annotation's own type args are directly the key/value
types.
===file===
<?php
/** @param Iterator<int, string> $items */
function fromIterator(Iterator $items): void {
    foreach ($items as $item) {
        strtoupper($item);
    }
}

/** @param IteratorAggregate<int, string> $items */
function fromIteratorAggregate(IteratorAggregate $items): void {
    foreach ($items as $item) {
        strtoupper($item);
    }
}

/** @param Traversable<int, string> $items */
function fromTraversable(Traversable $items): void {
    foreach ($items as $item) {
        strtoupper($item);
    }
}
===expect===
