===description===
Spreading a single, closed, string-keyed shape into an array literal
(the `[...$defaults, ...$overrides]` config-merge idiom) preserves the
precise shape instead of widening the whole literal to a generic array.
An int-keyed spread source still falls back (renumbering isn't modeled).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param array{a: int, b: string} $defaults */
function test_spread_preserves_shape(array $defaults): void {
    $merged = ['c' => true, ...$defaults];
    /** @mir-check $merged is array{c: true, a: int, b: string} */
    $_ = $merged;
}

/** @param array{a: int} $first
 *  @param array{b: string} $second
 */
function test_two_spreads_merge(array $first, array $second): void {
    $merged = [...$first, ...$second];
    /** @mir-check $merged is array{a: int, b: string} */
    $_ = $merged;
}

/** @param array{a: int, b: string} $defaults */
function test_later_key_overwrites_spread(array $defaults): void {
    $merged = [...$defaults, 'a' => 'overridden'];
    /** @mir-check $merged is array{a: "overridden", b: string} */
    $_ = $merged;
}

/** @param list<int> $items */
function test_int_keyed_spread_falls_back(array $items): void {
    $merged = ['c' => true, ...$items];
    /** @mir-check $merged is array<"c"|int, true|int> */
    $_ = $merged;
}
===expect===
