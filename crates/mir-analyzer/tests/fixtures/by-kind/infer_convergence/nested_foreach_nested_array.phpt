===description===
Nested foreach building a nested array must converge: inner list construction
inside an outer keyed-array loop must not cause unbounded union growth across
Salsa fixpoint iterations.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array<string, list<int>> $groups
 * @return array<string, int>
 */
function sumGroups(array $groups): array {
    $sums = [];
    foreach ($groups as $key => $values) {
        $total = 0;
        foreach ($values as $val) {
            $total += $val;
        }
        $sums[$key] = $total;
    }
    return $sums;
}
===expect===
