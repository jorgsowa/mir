===description===
`arraylike-object<K, V>` resolves structurally to
`ArrayAccess<K, V>&Countable&Traversable<K, V>`, so a matching object keeps
its indexed value type, foreach key/value types, and countability.
===config===
suppress=MissingConstructor,UnusedParam,UnusedVariable
===file===
<?php
/**
 * @template TKey
 * @template TValue
 * @implements ArrayAccess<TKey, TValue>
 * @implements IteratorAggregate<TKey, TValue>
 */
class Bag implements ArrayAccess, Countable, IteratorAggregate {
    public function offsetExists(mixed $offset): bool { return true; }
    public function offsetGet(mixed $offset): mixed { return null; }
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
    public function count(): int { return 0; }
    public function getIterator(): Traversable { yield 'x' => 1; }
}

/** @extends Bag<string, int> */
final class StringIntBag extends Bag {}

/** @param int $x */
function takesInt($x): void {}

/** @param string $x */
function takesString($x): void {}

/** @param arraylike-object<string, int> $bag */
function usesArraylike($bag): void {
    $value = $bag['foo'];
    takesInt($value);

    foreach ($bag as $key => $item) {
        takesString($key);
        takesInt($item);
    }

    count($bag);
}

usesArraylike(new StringIntBag());
===expect===
