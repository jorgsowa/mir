===description===
The `arraylike-object<K, V>` spelling is only meaningful in its generic form,
and those key/value args must participate in subtype matching rather than
being treated like an untyped "some object".
===config===
suppress=MissingConstructor,UnusedParam
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
    public function getIterator(): Traversable { yield 1 => 'x'; }
}

/** @extends Bag<int, string> */
final class IntStringBag extends Bag {}

/** @param arraylike-object<string, int> $bag */
function takesArraylike($bag): void {}

takesArraylike(new IntStringBag());
===expect===
InvalidArgument@23:15-23:33: Argument $bag of takesArraylike() expects 'ArrayAccess<string, int>&Countable&Traversable<string, int>', got 'IntStringBag'
