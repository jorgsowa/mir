===description===
Closures are objects and can never be loosely equal to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\Closure $fn): void {
    if ($fn == null) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:19: '==' between 'Closure' and 'null' is always false — these types can never be loosely equal
RedundantCondition@3:8-3:19: Condition is always true/false for type 'bool'
