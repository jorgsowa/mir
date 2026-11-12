===description===
A variable assigned under `count($a) > n` is defined wherever a count at least as large holds again (`count($a) > n`, `count($a) >= n + 1`, `n < count($a)`). A smaller bound or a write to the array keeps it possibly undefined.
===file===
<?php
/** @param list<int> $ids */
function sameCount(array $ids, bool $verbose): void {
    if (count($ids) > 1) { $keys = array_keys($ids); }
    if ($verbose) { echo 1; }
    if (count($ids) > 1) {
        foreach ($keys as $key) { echo $key; }
    }
}

/** @param list<int> $ids */
function equivalentForms(array $ids): void {
    if (count($ids) >= 3) { $keys = array_keys($ids); }
    if (2 < count($ids)) {
        foreach ($keys as $key) { echo $key; }
    }
}

/** @param list<int> $ids */
function smallerBound(array $ids): void {
    if (count($ids) > 1) { $keys = array_keys($ids); }
    if (count($ids) > 0) {
        foreach ($keys as $key) { echo $key; }
//               ^^^^^ PossiblyUndefinedVariable: Variable $keys might not be defined
    }
}

/** @param list<int> $ids */
function arrayWritten(array $ids): void {
    if (count($ids) > 1) { $keys = array_keys($ids); }
    $ids[] = 1;
    if (count($ids) > 1) {
        foreach ($keys as $key) { echo $key; }
//               ^^^^^ PossiblyUndefinedVariable: Variable $keys might not be defined
    }
}
