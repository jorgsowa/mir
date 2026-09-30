===description===
reports redundant check after narrowing
===file===
<?php
function f(string|int $x): void {
    if (is_string($x)) {
        if (is_string($x)) {}
//          ^^^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
    }
}
===expect===
