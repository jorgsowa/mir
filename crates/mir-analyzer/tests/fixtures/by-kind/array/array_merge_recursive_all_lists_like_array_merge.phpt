===description===
array_merge_recursive on all-list arguments has no possible int-key
collision (array_merge always renumbers int keys regardless), so it
behaves identically to array_merge: re-indexed from 0, value types
unioned, non-empty if any argument is non-empty. The general string-keyed
recursive-merge case (colliding scalars wrap, colliding arrays deep-merge)
isn't modeled — falls back to the generic stub for non-list arguments.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<int> $ints
 * @param list<string> $maybe_empty_strings
 * @param array<string, int> $assoc
 */
function test(array $ints, array $maybe_empty_strings, array $assoc): void {
    $merged = array_merge_recursive($ints, $maybe_empty_strings);
    /** @mir-check $merged is non-empty-list<int|string> */
    $_ = $merged;

    $with_assoc = array_merge_recursive($ints, $assoc);
    /** @mir-check $with_assoc is array */
    $_ = $with_assoc;
}
===expect===
