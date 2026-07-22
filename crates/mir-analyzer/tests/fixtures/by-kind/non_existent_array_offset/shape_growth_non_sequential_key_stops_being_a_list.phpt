===description===
Writing a non-sequential key (`$arr[5] = …`) onto a proven-empty `list<int>`
still grows the shape to `array{5: int}`, but the result is no longer
list-shaped — indexing `$arr[0]` afterward is flagged as a non-existent
offset instead of silently being treated as in-bounds.
===config===
suppress=UnusedParam
===file===
<?php
/** @param list<int> $arr */
function test(array $arr): int {
    if ($arr === []) {
        $arr[5] = 1;
        return $arr[0];
    }
    return 0;
}
===expect===
NonExistentArrayOffset@6:20-6:21: Array offset '0' does not exist
MixedReturnStatement@6:8-6:23: Cannot return a mixed type from function with declared return type 'int'
