===description===
An array literal with elements both before and after a spread (or before
and after a key that doesn't resolve to a single literal) must still merge
every element's type into the fallback array — the element after the
spread/dynamic-key point isn't dropped, and no element is analyzed twice.
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
    /** @mir-check $merged is array<int|string, int|"z"> */
    $_ = $merged;

    $withDynamicKey = [$before, $dynamicKey => 'mid', $after];
    /** @mir-check $withDynamicKey is array<int|string, 1|"mid"|"z"> */
    $_ = $withDynamicKey;
}
===expect===
