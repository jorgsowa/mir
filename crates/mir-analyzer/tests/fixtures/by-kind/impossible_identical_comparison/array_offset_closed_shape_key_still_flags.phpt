===description===
Null comparisons on required shape keys remain impossible.
===file===
<?php
/** @param array{a: int} $shape */
function test(array $shape): void {
    $v = $shape['a'];
    if ($v !== null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '!==' between 'int' and 'null' is always true — these types can never be identical
//      ^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
}
===expect===
