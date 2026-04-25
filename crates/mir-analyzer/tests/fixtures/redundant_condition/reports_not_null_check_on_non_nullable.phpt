===file===
<?php
function f(string $x): void {
    if ($x !== null) {}
}
===expect===
RedundantCondition: Condition is always true/false for type 'bool'
