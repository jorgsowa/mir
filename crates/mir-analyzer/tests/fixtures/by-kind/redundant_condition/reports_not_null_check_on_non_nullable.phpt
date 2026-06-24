===description===
reports not null check on non nullable
===file===
<?php
function f(string $x): void {
    if ($x !== null) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:19: '!==' between 'string' and 'null' is always true — these types can never be identical
RedundantCondition@3:8-3:19: Condition is always true/false for type 'bool'
