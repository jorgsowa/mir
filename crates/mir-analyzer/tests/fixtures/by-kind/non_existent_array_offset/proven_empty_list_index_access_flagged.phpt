===description===
Once `$values === []` proves a `list<T>` parameter empty, indexing it
(`$values[0]`) inside that branch is flagged as a non-existent offset — the
empty branch narrows to the same closed, zero-property shape an empty `[]`
literal itself has, rather than leaving the pre-narrow element type
(and its now-stale index) in place.
===config===
suppress=UnusedParam
===file===
<?php
/** @param list<int> $values */
function rejectsEmptyBranch(array $values): int {
    if ($values === []) {
        return $values[0];
//      ^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                     ^ NonExistentArrayOffset: Array offset '0' does not exist
    }

    return $values[0];
}
===expect===
