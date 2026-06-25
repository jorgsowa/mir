===description===
do-while loop building a keyed array must converge without infinite type union growth.
Exercises the same widen_array_with_value_and_key accumulation path as foreach,
but through the do-while control-flow shape.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @return array<string, int>
 */
function buildMapping(int $start, int $end): array {
    $map = [];
    $i = $start;
    do {
        $map[(string) $i] = $i * 2;
        $i++;
    } while ($i <= $end);
    return $map;
}
===expect===
