===description===
The != operator is also checked: object != null is always true.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj): void {
    if ($obj != null) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:20: '!=' between 'stdClass' and 'null' is always true — these types can never be loosely equal
RedundantCondition@3:8-3:20: Condition is always true/false for type 'bool'
