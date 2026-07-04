===description===
MissingClosureReturnType fires for closures passed directly as function arguments,
not just for closures assigned to variables.
===file===
<?php
$result = array_filter([1, 2, 3], function(int $x) {
    return $x > 1;
});
===expect===
UnusedVariable@2:0-2:7: Variable $result is never read
MissingClosureReturnType@2:34-4:1: Closure has no return type annotation
