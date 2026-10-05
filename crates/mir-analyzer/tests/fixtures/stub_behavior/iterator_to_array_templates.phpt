===description===
iterator_to_array() keeps the iterator's key and value types.
===file===
<?php
/** @implements IteratorAggregate<string, int> */
final class TypedBag implements IteratorAggregate {
    public function getIterator(): Traversable {
        return new ArrayIterator([]);
    }
}

final class Plain implements IteratorAggregate {
    public function getIterator(): Traversable {
        return new ArrayIterator([]);
    }
}

function fromBag(TypedBag $bag): array {
    $r = iterator_to_array($bag);
    /** @mir-check $r is array<string, int> */
    return $r;
}

/** @param Traversable<string, int> $t */
function fromTraversable(Traversable $t): array {
    $r = iterator_to_array($t);
    /** @mir-check $r is array<string, int> */
    return $r;
}

/** @param array<string, int> $a */
function fromArray(array $a): array {
    $r = iterator_to_array($a);
    /** @mir-check $r is array<string, int> */
    return $r;
}

/** @param Generator<string, int> $g */
function fromGenerator(Generator $g): array {
    $r = iterator_to_array($g, true);
    /** @mir-check $r is array<string, int> */
    return $r;
}

function dropKeys(TypedBag $bag): array {
    $r = iterator_to_array($bag, false);
    /** @mir-check $r is list<int> */
    return $r;
}

function fromPlain(Plain $plain): array {
    $r = iterator_to_array($plain);
    /** @mir-check $r is array */
    return $r;
}

function fromIterable(iterable $any): array {
    $r = iterator_to_array($any);
    /** @mir-check $r is array */
    return $r;
}

/** @param Generator<non-empty-list<int>> $g */
function valueOnlyGenerator(Generator $g): array {
    $r = iterator_to_array($g, false);
    /** @mir-check $r is list<non-empty-list<int>> */
    return $r;
}

final class Consumer {
    /** @var list<string> */
    private array $keys = [];

    public function load(TypedBag $bag): void {
        $this->keys = array_keys(iterator_to_array($bag));
    }
}
===expect===
