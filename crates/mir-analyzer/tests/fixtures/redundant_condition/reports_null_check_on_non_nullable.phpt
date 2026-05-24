===description===
reports null check on non nullable
===file===
<?php
function f(string $x): void {
    if ($x === null) {}
}
===expect===
RedundantCondition@3:9: Condition is always true/false for type 'bool'
