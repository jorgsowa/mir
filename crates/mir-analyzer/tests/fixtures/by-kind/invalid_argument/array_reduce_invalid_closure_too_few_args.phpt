===description===
Array reduce invalid closure too few args
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$arr = [2, 3, 4, 5];

$direct_closure_result = array_reduce(
    $arr,
    function() : int {
        return 5;
    },
    1
);
===expect===
InvalidArgument@6:4-8:5: Argument $callback of array_reduce() expects 'callable accepting at least 2 arguments', got 'callable accepting 0 arguments'
