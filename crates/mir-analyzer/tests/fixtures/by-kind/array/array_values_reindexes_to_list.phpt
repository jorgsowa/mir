===description===
array_values re-indexes any array to a sequential integer list.
Non-emptiness is preserved: a non-empty source yields a non-empty-list,
a possibly-empty source yields a list. The key type is always discarded;
only the value type is carried into the list element type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-array<string, int> $ne_assoc
 * @param array<string, int> $assoc
 */
function test(array $ne_assoc, array $assoc): void {
    // non-empty-array<string, int> → non-empty-list<int>
    $ne_vals = array_values($ne_assoc);
    /** @mir-check $ne_vals is non-empty-list<int> */
    $_ = $ne_vals;

    // array<string, int> → list<int>
    $vals = array_values($assoc);
    /** @mir-check $vals is list<int> */
    $_ = $vals;
}
===expect===
