===description===
reports is string on string type
===file===
<?php
function f(string $x): void {
    if (is_string($x)) {}
}
===expect===
RedundantCondition@3:9-3:22: Condition is always true/false for type 'bool'
