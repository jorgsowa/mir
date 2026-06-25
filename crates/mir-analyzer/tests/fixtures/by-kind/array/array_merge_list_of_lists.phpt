===description===
array_merge of two list-typed arguments produces a list whose element type is the
union of both element types. The result is non-empty if at least one argument is
provably non-empty; otherwise possibly-empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<int> $ne
 * @param list<int> $maybe_empty
 */
function test(array $ne, array $maybe_empty): void {
    // non-empty + maybe-empty → non-empty-list
    $merged_ne = array_merge($ne, $maybe_empty);
    /** @mir-check $merged_ne is non-empty-list<int> */
    $_ = $merged_ne;

    // maybe-empty + non-empty → non-empty-list (non-empty from second arg)
    $merged_ne2 = array_merge($maybe_empty, $ne);
    /** @mir-check $merged_ne2 is non-empty-list<int> */
    $_ = $merged_ne2;

    // maybe-empty + maybe-empty → list (no non-empty guarantee)
    $merged_maybe = array_merge($maybe_empty, $maybe_empty);
    /** @mir-check $merged_maybe is list<int> */
    $_ = $merged_maybe;
}
===expect===
