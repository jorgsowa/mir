===description===
While loop appending to a list must converge: widen_array_as_list must not produce
unbounded union growth across fixpoint iterations.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param list<string> $items
 * @return list<string>
 */
function filterNonEmpty(array $items): array {
    $result = [];
    $i = 0;
    while ($i < count($items)) {
        if ($items[$i] !== '') {
            $result[] = $items[$i];
        }
        $i++;
    }
    return $result;
}
===expect===
