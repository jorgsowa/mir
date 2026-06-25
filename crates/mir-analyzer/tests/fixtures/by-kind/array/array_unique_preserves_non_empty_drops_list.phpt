===description===
array_unique preserves non-emptiness (a non-empty input always has at least one
distinct value) but drops the list guarantee: deduplication can leave gaps in
integer keys, so the result is a plain array (not list), though non-empty when
the source is non-empty. Key and value types from the source are preserved.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<string> $ne
 * @param list<string> $maybe_empty
 */
function test(array $ne, array $maybe_empty): void {
    // non-empty-list<string> → non-empty-array<int, string> (not list; non-empty preserved)
    $unique_ne = array_unique($ne);
    /** @mir-check $unique_ne is non-empty-array<int, string> */
    $_ = $unique_ne;

    // list<string> → array<int, string> (not list; possibly-empty)
    $unique = array_unique($maybe_empty);
    /** @mir-check $unique is array<int, string> */
    $_ = $unique;
}
===expect===
