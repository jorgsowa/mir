===description===
Null comparisons on required shape keys remain impossible.
===file===
<?php
/** @param array{a: int} $shape */
function test(array $shape): void {
    $v = $shape['a'];
    if ($v !== null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'int' and 'null' is always true — these types can never be identical
//      ^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
}
===expect===
