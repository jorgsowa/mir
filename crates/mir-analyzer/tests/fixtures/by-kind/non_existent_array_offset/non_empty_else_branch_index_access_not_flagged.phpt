===description===
Regression guard: the `else`/non-empty branch of `$values === []` (where
`$values` narrows to `non-empty-list<int>`) still permits indexing at 0 —
only the proven-empty branch should gain the new `NonExistentArrayOffset`.
===config===
suppress=UnusedParam
===file===
<?php
/** @param list<int> $values */
function acceptsNonEmptyBranch(array $values): int {
    if ($values === []) {
        return 0;
    }

    return $values[0];
}
===expect===
