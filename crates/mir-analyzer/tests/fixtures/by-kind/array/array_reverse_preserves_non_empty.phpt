===description===
array_reverse preserves non-emptiness: reversing a non-empty list is still non-empty.
The result is always a list (integer re-indexed) regardless of whether the source
was an associative array or a list. A possibly-empty source stays possibly-empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<string> $ne
 * @param list<string> $maybe_empty
 */
function test(array $ne, array $maybe_empty): void {
    // non-empty-list<string> → non-empty-list<string>
    $rev_ne = array_reverse($ne);
    /** @mir-check $rev_ne is non-empty-list<string> */
    $_ = $rev_ne;

    // list<string> → list<string>
    $rev = array_reverse($maybe_empty);
    /** @mir-check $rev is list<string> */
    $_ = $rev;
}
===expect===
