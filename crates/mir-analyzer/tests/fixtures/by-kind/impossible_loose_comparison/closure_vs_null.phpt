===description===
Closures are objects and can never be loosely equal to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\Closure $fn): void {
    if ($fn == null) {}
//      ^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'Closure' and 'null' is always false — these types can never be loosely equal
//      ^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
}
===expect===
