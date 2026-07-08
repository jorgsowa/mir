===description===
FN: a literal array (`[1, 2, 3]`) is represented as `TKeyedArray{is_list:
true}`, not `TList`/`TNonEmptyList` — array_map's list/non-empty detection
only matched the latter two, so mapping over a literal list lost list-ness
and fell back to a generic `array<int, T>`.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
$arr = [1, 2, 3];
$mapped = array_map(function (int $x) {
    return $x * 2;
}, $arr);
/** @mir-check $mapped is non-empty-list<int> */
$_ = $mapped;
===expect===
