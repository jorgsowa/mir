===source===
<?php
function f(string $x): void {
    if (is_string($x)) {}
}
===expect===
RedundantCondition: Condition is always true/false for type 'bool'
