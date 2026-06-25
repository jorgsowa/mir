===description===
array_pop returns the element type directly (without |null) when the array is
provably non-empty; returns T|null when the array is possibly-empty because
PHP returns null when the array is empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<string> $ne
 * @param list<string> $maybe_empty
 */
function test(array $ne, array $maybe_empty): void {
    // non-empty source → string (no null; PHP always pops one element)
    $popped_ne = array_pop($ne);
    /** @mir-check $popped_ne is string */
    $_ = $popped_ne;

    // possibly-empty source → string|null (PHP returns null when array is empty)
    $popped = array_pop($maybe_empty);
    /** @mir-check $popped is string|null */
    $_ = $popped;
}
===expect===
