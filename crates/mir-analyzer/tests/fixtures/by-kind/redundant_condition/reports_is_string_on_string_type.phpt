===description===
reports is string on string type
===file===
<?php
function f(string $x): void {
    if (is_string($x)) {}
//      ^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}
===expect===
