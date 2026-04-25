===file===
<?php
function f(string|int $x): void {
    if (is_string($x)) {
        if (is_string($x)) {}
    }
}
===expect===
RedundantCondition: Condition is always true/false for type 'bool'
