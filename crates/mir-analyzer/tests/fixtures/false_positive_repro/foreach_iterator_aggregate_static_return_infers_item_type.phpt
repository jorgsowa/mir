===description===
`getIterator(): static { return $this; }` — a common self-iterating
IteratorAggregate+Iterator class — resolves the loop's item type through the
class's own Iterator implementation instead of leaking `mixed` because
`static` is a `TStaticObject`, not a `TNamedObject`, atom.
===config===
suppress=MissingPropertyType,UnusedParam,UnusedForeachValue
===file===
<?php
/** @template T */
class Bag implements IteratorAggregate, Iterator {
    /** @var list<T> */
    private array $items = [];
    private int $pos = 0;

    /** @param T $item */
    public function add($item): void {
        $this->items[] = $item;
    }

    public function getIterator(): static {
        return $this;
    }

    /** @return T */
    public function current(): mixed { return $this->items[$this->pos]; }
    public function key(): int { return $this->pos; }
    public function next(): void { $this->pos++; }
    public function rewind(): void { $this->pos = 0; }
    public function valid(): bool { return isset($this->items[$this->pos]); }
}

class Animal {}

/** @extends Bag<Animal> */
class AnimalBag extends Bag {}

function sumAnimals(AnimalBag $bag): void {
    foreach ($bag as $x) {
        /** @mir-check $x is Animal */
        $_ = 1;
    }
}
===expect===
