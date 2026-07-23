===description===
Reading a literal key off a heterogeneous array union (shape alongside a
generic array/list/string) merges every union member's contribution
instead of returning only the first matching atom's — a shape member no
longer silently drops a co-existing array<K,V>/list<V> alternative's value
type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param array{a: int}|array<string, string> $x */
function test_shape_and_array(array $x): void {
    /** @mir-check $x['a'] is int|string */
    $_ = $x['a'];
}

/** @param array{0: int}|list<string> $x */
function test_shape_and_list(array $x): void {
    /** @mir-check $x[0] is int|string */
    $_ = $x[0];
}

/** @param array{a: int} $x */
function test_pure_shape_unaffected(array $x): void {
    /** @mir-check $x['a'] is int */
    $_ = $x['a'];
}

/** @param array<string, int> $x */
function test_pure_generic_unaffected(array $x): void {
    /** @mir-check $x['a'] is int */
    $_ = $x['a'];
}
===expect===
