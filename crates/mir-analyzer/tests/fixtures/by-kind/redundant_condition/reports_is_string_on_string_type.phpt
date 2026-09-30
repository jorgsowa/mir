===description===
reports is string on string type
===file===
<?php
function f(string $x): void {
    if (is_string($x)) {}
//      ^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
}
===expect===
