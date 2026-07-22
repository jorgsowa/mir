===description===
The same proven-empty narrowing applies to a plain associative
`array<string, T>` (not just `list<T>`) — a string-key access inside the
`=== []` branch is flagged the same way an int-key access on an empty list
is.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array<string, int> $counts */
function rejectsEmptyBranch(array $counts): int {
    if ($counts === []) {
        return $counts['total'];
    }

    return $counts['total'];
}
===expect===
MixedReturnStatement@5:8-5:32: Cannot return a mixed type from function with declared return type 'int'
NonExistentArrayOffset@5:23-5:30: Array offset 'total' does not exist
