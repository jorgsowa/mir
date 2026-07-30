===description===
A typed catch-all remainder (`...array<K, V>`) only excuses UNKNOWN extra keys — a
key the shape actually declares must still satisfy its own declared value type.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array{a: int, ...array<string, int>} $x */
function wantsShape(array $x): void {}

wantsShape(['a' => 'not-an-int']);
===expect===
InvalidArgument@5:11-5:32: Argument $x of wantsShape() expects 'array{'a': int}', got 'array{'a': "not-an-int"}'
