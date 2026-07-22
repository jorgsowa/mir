===description===
array_diff/array_intersect return a subset of the FIRST argument's own
entries (by value, key, or both) — never altering surviving entries' keys or
values. So the result's key/value types are exactly the first argument's.
Never provably non-empty (everything could be filtered out) and never a
list (original, possibly non-sequential, int keys are preserved verbatim).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param array<string, int> $assoc
 * @param non-empty-array<string, int> $ne_assoc
 */
function test(array $assoc, array $ne_assoc): void {
    $diffed = array_diff($assoc, ['x' => 1]);
    /** @mir-check $diffed is array<string, int> */
    $_ = $diffed;

    // Source non-emptiness is NOT preserved: everything could be filtered out.
    $diffed_ne_source = array_diff($ne_assoc, ['a' => 1]);
    /** @mir-check $diffed_ne_source is array<string, int> */
    $_ = $diffed_ne_source;

    $intersected = array_intersect($assoc, ['x' => 1]);
    /** @mir-check $intersected is array<string, int> */
    $_ = $intersected;
}
===expect===
