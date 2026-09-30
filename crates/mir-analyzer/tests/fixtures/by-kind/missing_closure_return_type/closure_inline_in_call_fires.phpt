===description===
MissingClosureReturnType fires for closures passed directly as function arguments,
not just for closures assigned to variables.
===file===
<?php
$result = array_filter([1, 2, 3], function(int $x) {
//<^^^^^^^ UnusedVariable: Variable $result is never read
//                                ^ +2:1 MissingClosureReturnType: Closure has no return type annotation
    return $x > 1;
});
===expect===
