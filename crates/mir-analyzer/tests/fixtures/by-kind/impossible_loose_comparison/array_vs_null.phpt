===description===
Arrays can never be loosely equal to null in PHP — [] == null is false.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $arr): void {
    if ($arr == null) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:20: '==' between 'array<mixed, mixed>' and 'null' is always false — these types can never be loosely equal
RedundantCondition@3:8-3:20: Condition is always true/false for type 'bool'
