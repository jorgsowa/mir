===description===
reports redundant check after narrowing
===file===
<?php
function f(string|int $x): void {
    if (is_string($x)) {
        if (is_string($x)) {}
    }
}
===expect===
RedundantCondition@4:13-4:26: Condition is always true/false for type 'bool'
