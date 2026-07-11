===description===
FP: `$obj[$idx]` on an `ArrayAccess`-implementing receiver always yielded
`mixed`, regardless of the class's `offsetGet()` return type — and the
offset expression itself was checked against the plain-PHP-array
"offset must be an array-key" rule, which doesn't apply to `ArrayAccess`
objects (they accept whatever type their own `offsetGet`/`offsetSet`
declare, e.g. the SPL `WeakMap` is keyed by `object`). Now the value type
is resolved from an `@implements ArrayAccess<TKey, TValue>` annotation
(substituting the receiver's own concrete type args) or, absent that,
from `offsetGet()`'s resolved return type, and the object-offset case no
longer trips the array-key check.
===config===
suppress=MixedArrayOffset,UnusedParam
===file===
<?php
class IntList implements ArrayAccess {
    /** @var list<int> */
    private array $items = [];
    public function offsetExists(mixed $offset): bool { return isset($this->items[$offset]); }
    public function offsetGet(mixed $offset): int { return $this->items[$offset]; }
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

function readInt(IntList $list): int {
    return $list[0] + 1;
}

/**
 * @template T
 * @implements ArrayAccess<int, T>
 */
final class TypedList implements ArrayAccess {
    /** @var list<T> */
    private array $items = [];
    public function offsetExists(mixed $offset): bool { return isset($this->items[$offset]); }
    /** @return T */
    public function offsetGet(mixed $offset): mixed { return $this->items[$offset]; }
    /** @param T $value */
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

/** @param TypedList<int> $list */
function readTyped(TypedList $list): int {
    return $list[0] + 1;
}

function weakMapObjectOffset(WeakMap $map, object $key): bool {
    return isset($map[$key]);
}
===expect===
