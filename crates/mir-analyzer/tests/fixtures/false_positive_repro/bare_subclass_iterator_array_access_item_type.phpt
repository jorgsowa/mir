===description===
foreach and `$obj[$idx]` item-type resolution for a class implementing
Iterator/ArrayAccess must still substitute the receiver's own type args
when the class is a totally bare subclass (no own `@template` at all) of
the class that carries the `@implements Iterator<TKey,TValue>`/
`ArrayAccess<TKey,TValue>` annotation — both resolvers looked up the
receiver's own-only template params to build the substitution map, which
is empty for a bare subclass, so the item type leaked through unresolved.
===config===
suppress=MissingConstructor,MixedArrayOffset,UnusedParam
===file===
<?php
/**
 * @template T
 * @implements Iterator<int, T>
 */
class Bag implements Iterator {
    /** @var list<T> */
    private array $items = [];
    private int $pos = 0;
    /** @return T */
    public function current(): mixed { return $this->items[$this->pos]; }
    public function key(): int { return $this->pos; }
    public function next(): void { $this->pos++; }
    public function rewind(): void { $this->pos = 0; }
    public function valid(): bool { return isset($this->items[$this->pos]); }
}

class IntBag extends Bag {}

/** @param IntBag<int> $bag */
function sumIntBag(IntBag $bag): int {
    $total = 0;
    foreach ($bag as $item) {
        $total += $item;
    }
    return $total;
}

/**
 * @template T
 * @implements ArrayAccess<int, T>
 */
class TypedList implements ArrayAccess {
    /** @var list<T> */
    private array $items = [];
    public function offsetExists(mixed $offset): bool { return isset($this->items[$offset]); }
    /** @return T */
    public function offsetGet(mixed $offset): mixed { return $this->items[$offset]; }
    /** @param T $value */
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

class IntTypedList extends TypedList {}

/** @param IntTypedList<int> $list */
function readIntTypedList(IntTypedList $list): int {
    return $list[0] + 1;
}
===expect===
