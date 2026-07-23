===description===
Destructuring assignment ([$a] = $x / ['a' => $a] = $x) merges every union
member's contribution — a shape member no longer silently drops a
co-existing array<K,V>/list<V> alternative's value type, same fix as plain
array-access reads.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param array{a: int}|array<string, string> $x */
function test_keyed_destructure(array $x): void {
    ['a' => $a] = $x;
    /** @mir-check $a is int|string */
    $_ = $a;
}

/** @param array<string, int>|list<string> $x */
function test_positional_destructure(array $x): void {
    [$a] = $x;
    /** @mir-check $a is int|string */
    $_ = $a;
}

/** @param array{a: int} $x */
function test_pure_shape_unaffected(array $x): void {
    ['a' => $a] = $x;
    /** @mir-check $a is int */
    $_ = $a;
}

/** @param array<string, int> $x */
function test_pure_generic_unaffected(array $x): void {
    ['a' => $a] = $x;
    /** @mir-check $a is int */
    $_ = $a;
}
===expect===
