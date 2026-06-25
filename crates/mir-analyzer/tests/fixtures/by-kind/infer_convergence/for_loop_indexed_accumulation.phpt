===description===
For loop building an indexed array must converge: widen_array_with_value_and_key
must not produce unbounded union growth when integer indices are assigned in a
classic for loop.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param list<int> $values
 * @return array<int, string>
 */
function buildLabels(array $values): array {
    $result = [];
    for ($i = 0; $i < count($values); $i++) {
        $result[$i] = 'item_' . (string) $values[$i];
    }
    return $result;
}
===expect===
