===description===
Null comparisons on required shape keys remain impossible.
===file===
<?php
/** @param array{a: int} $shape */
function test(array $shape): void {
    $v = $shape['a'];
    if ($v !== null) {}
}
===expect===
ImpossibleIdenticalComparison@5:8-5:19: '!==' between 'int' and 'null' is always true — these types can never be identical
RedundantCondition@5:8-5:19: Condition is always true/false for type 'bool'
