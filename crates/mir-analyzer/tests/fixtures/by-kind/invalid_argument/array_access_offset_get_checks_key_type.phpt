===description===
`$obj[$idx]` on an ArrayAccess-implementing receiver checks `$idx` against
the receiver's declared TKey (from `@implements ArrayAccess<TKey, TValue>`)
— previously only the value type was resolved, never the key.
===config===
suppress=UnusedParam,MissingConstructor,UnusedVariable
===file===
<?php
/**
 * @template TKey
 * @template TValue
 * @implements ArrayAccess<TKey, TValue>
 */
class TypedMap implements ArrayAccess {
    public function offsetExists(mixed $offset): bool { return false; }
    public function offsetGet(mixed $offset): mixed { return null; }
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

/** @param TypedMap<string, int> $m */
function reads(TypedMap $m): void {
    $a = $m['x'];
    $b = $m[42];
}
===expect===
InvalidArgument@17:12-17:14: Argument $offset of offsetGet() expects 'string', got '42'
