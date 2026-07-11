===description===
FP: `foreach` over an object — a `Generator`, or a user-defined class
implementing `Iterator`/`IteratorAggregate` — always produced `mixed`
key/value types, regardless of the object's declared item types. This meant
every read of the loop variable (and anything derived from it) was flagged
as mixed, even when the class fully typed its iteration:
  - `Generator<TKey, TValue>` (from a generator function's return type)
  - a class with `@template T` + `@implements Iterator<int, T>`
  - a class with `@template T` + `@implements IteratorAggregate<int, T>`
  - a plain (non-generic) class whose `current()`/`key()` are natively typed
Now the key/value types are resolved from the class's generic `@implements`
annotation (substituting the receiver's own type args), or — absent that
annotation — from `current()`/`key()`'s resolved return types, recursing
through `getIterator()` for `IteratorAggregate`.
===file===
<?php
/** @return Generator<int, string> */
function genValues(): Generator {
    yield 'a';
    yield 'b';
}

function sumLengths(): int {
    $total = 0;
    foreach (genValues() as $key => $value) {
        $total += $key + strlen($value);
    }
    return $total;
}

/**
 * @template T
 * @implements Iterator<int, T>
 */
final class Bag implements Iterator {
    /** @var list<T> */
    private array $items = [];
    private int $pos = 0;

    /** @param T $item */
    public function add(mixed $item): void {
        $this->items[] = $item;
    }

    public function current(): mixed { return $this->items[$this->pos]; }
    public function key(): int { return $this->pos; }
    public function next(): void { $this->pos++; }
    public function rewind(): void { $this->pos = 0; }
    public function valid(): bool { return isset($this->items[$this->pos]); }
}

/** @param Bag<int> $bag */
function sumBag(Bag $bag): int {
    $total = 0;
    foreach ($bag as $item) {
        $total += $item;
    }
    return $total;
}

/**
 * @template T
 * @implements IteratorAggregate<int, T>
 */
final class Box implements IteratorAggregate {
    /** @var list<T> */
    private array $items = [];

    /** @return Iterator<int, T> */
    public function getIterator(): Iterator {
        return new Bag();
    }
}

/** @param Box<int> $box */
function sumBox(Box $box): int {
    $total = 0;
    foreach ($box as $item) {
        $total += $item;
    }
    return $total;
}

class PlainIntBag implements Iterator {
    /** @var list<int> */
    private array $items = [];
    private int $pos = 0;

    public function current(): int { return $this->items[$this->pos]; }
    public function key(): int { return $this->pos; }
    public function next(): void { $this->pos++; }
    public function rewind(): void { $this->pos = 0; }
    public function valid(): bool { return isset($this->items[$this->pos]); }
}

function sumPlainBag(PlainIntBag $bag): int {
    $total = 0;
    foreach ($bag as $item) {
        $total += $item;
    }
    return $total;
}
===expect===
