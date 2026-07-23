===description===
An array literal with elements both before and after a spread of a
string-keyed shape merges into a precise shape (int keys renumbered around
the spread's own string keys) — the element after the spread isn't
dropped, and no element is analyzed twice. A key that doesn't resolve to a
single literal still forces the generic-array fallback, merging every
element's type instead.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{a: int} $x
 */
function test(array $x, string $dynamicKey): void {
    $before = 1;
    $after = 'z';
    $merged = [$before, ...$x, $after];
    /** @mir-check $merged is array{0: 1, a: int, 1: "z"} */
    $_ = $merged;

    $withDynamicKey = [$before, $dynamicKey => 'mid', $after];
    /** @mir-check $withDynamicKey is array<int|string, 1|"mid"|"z"> */
    $_ = $withDynamicKey;
}
===expect===
