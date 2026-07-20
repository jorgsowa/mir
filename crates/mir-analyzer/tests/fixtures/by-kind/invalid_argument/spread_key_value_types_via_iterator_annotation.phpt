===description===
`[...$bag]` (array spread) and `f(...$bag)` (argument spread) resolve a
spreadable object's real key/value types via its `@implements
Iterator<TKey, TValue>` annotation, not a naive `type_params[0]`/`[1]`
positional guess — `Bag<T>` here has only ONE own template (the value),
with the key fixed to `int` by the interface annotation itself, so a
positional guess over `Bag`'s own type_params would misread the single
`T` as the key instead of the value.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor,MissingPropertyType
===file===
<?php
/**
 * @template T
 * @implements Iterator<int, T>
 */
class Bag implements Iterator {
    /** @var T */
    private $value;
    public function current(): mixed { return $this->value; }
    public function key(): int { return 0; }
    public function next(): void {}
    public function rewind(): void {}
    public function valid(): bool { return false; }
}

/** @param Bag<string> $bag */
function spreadIntoArray(Bag $bag): void {
    $arr = [...$bag];
    /** @mir-check $arr is array<int, string> */
    $arr;
}

function takesStrings(string ...$values): void {}

/** @param Bag<string> $bag */
function spreadIntoArgs(Bag $bag): void {
    takesStrings(...$bag);
}
===expect===
