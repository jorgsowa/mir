===description===
A named generic type whose every type param is a literal, unconstrained
`mixed` (e.g. `Collection<mixed, mixed>`) carries no more information than
the bare class name and must display collapsed, same as Traversable<mixed,
mixed> collapsing to Traversable.
===file===
<?php
/**
 * @template TKey
 * @template TValue
 */
class Collection {}

/** @param Collection<mixed, mixed> $c */
function f($c): void { $_ = $c; }

function test(): void {
    f("hello");
}
===expect===
InvalidArgument@12:6-12:13: Argument $c of f() expects 'Collection', got '"hello"'
