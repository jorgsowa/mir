===description===
array_splice returns the array of removed elements. Unlike array_slice,
there's no preserve_keys parameter — int keys are always renumbered from 0
in the return value, so a list source's removed elements are also a list;
a string-keyed source preserves key/value types (not modeled as a list).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param list<int> $items
 * @param array<string, int> $assoc
 */
function test(array $items, array $assoc): void {
    $removed = array_splice($items, 1, 2);
    /** @mir-check $removed is list<int> */
    $_ = $removed;

    $removed_assoc = array_splice($assoc, 0, 1);
    /** @mir-check $removed_assoc is array<string, int> */
    $_ = $removed_assoc;
}
===expect===
