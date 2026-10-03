===description===
`$obj[$k] = $v` on an ArrayAccess-implementing receiver checks `$v` against
`offsetSet()`'s declared value-param type — previously any value was
silently accepted, since the write path only ever matched a plain array.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
    /** @param TValue $value */
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

/** @param TypedMap<string, int> $m */
function stores(TypedMap $m): void {
    $m['x'] = 1;
    $m['y'] = 'not an int';
//  ^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of offsetSet() expects 'int', got '"not an int"'
}
===expect===
