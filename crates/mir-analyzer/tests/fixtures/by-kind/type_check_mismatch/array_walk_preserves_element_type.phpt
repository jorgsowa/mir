===description===
array_walk()/array_walk_recursive() mutate values in place without adding,
removing, or reordering keys — the by-ref array keeps its original type
instead of collapsing to the stub's declared object|array.
===config===
suppress=UnusedVariable,UnusedParam,MissingClosureReturnType
===file===
<?php

/** @param list<int> $arr */
function test_array_walk_preserves_list(array $arr): void {
    array_walk($arr, function ($v) {
        echo $v;
    });
    /** @mir-check $arr is list<int> */
    $_ = $arr;
}

/** @param non-empty-array<string, int> $arr */
function test_array_walk_recursive_preserves_type(array $arr): void {
    array_walk_recursive($arr, function ($v) {
        echo $v;
    });
    /** @mir-check $arr is non-empty-array<string, int> */
    $_ = $arr;
}
===expect===
