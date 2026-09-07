===description===
foreach and `$obj[$idx]` item-type resolution both built the
class-template substitution map from only the receiver's own
`build_class_bindings` result, never merging in
`inherited_template_bindings` -- unlike 8+ other generic-substitution
call sites in this codebase. A RE-TEMPLATING subclass (its own
`@template T` + `@extends Ancestor<T>`, not a bare subclass) whose
ancestor carries the `@implements Iterator</ArrayAccess<TKey,TValue>`
annotation leaked the ancestor's raw template name (`TValue`) instead of
resolving it through the subclass's own binding.
===config===
suppress=MissingConstructor,MixedArrayOffset,UnusedParam
===file===
<?php
/**
 * @template TValue
 * @implements ArrayAccess<int, TValue>
 */
class Box implements ArrayAccess {
    /** @var list<TValue> */
    private array $items = [];
    public function offsetExists(mixed $offset): bool { return isset($this->items[$offset]); }
    /** @return TValue */
    public function offsetGet(mixed $offset): mixed { return $this->items[$offset]; }
    /** @param TValue $value */
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

/**
 * @template T
 * @extends Box<T>
 */
class NamedBox extends Box {}

/** @param NamedBox<int> $box */
function readNamedBox(NamedBox $box): int {
    return $box[0] + 1;
}

/**
 * @template TItem
 * @implements Iterator<int, TItem>
 */
class Bag implements Iterator {
    /** @var list<TItem> */
    private array $items = [];
    private int $pos = 0;
    /** @return TItem */
    public function current(): mixed { return $this->items[$this->pos]; }
    public function key(): int { return $this->pos; }
    public function next(): void { $this->pos++; }
    public function rewind(): void { $this->pos = 0; }
    public function valid(): bool { return isset($this->items[$this->pos]); }
}

/**
 * @template T
 * @extends Bag<T>
 */
class NamedBag extends Bag {}

/** @param NamedBag<int> $bag */
function sumNamedBag(NamedBag $bag): int {
    $total = 0;
    foreach ($bag as $item) {
        $total += $item;
    }
    return $total;
}
===expect===
