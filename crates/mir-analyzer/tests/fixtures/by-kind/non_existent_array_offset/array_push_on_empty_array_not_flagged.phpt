===description===
Regression guard: `array_push($arr, …)` onto a proven-empty array must be
recognized as adding an element — indexing the pushed value afterward must
not be flagged as a non-existent offset. Before the fix, the closed `array{}`
shape's zero properties gave `array_push`'s type inference nothing to fold,
so it silently left the variable typed as still-empty.
===config===
suppress=UnusedParam
===file===
<?php
function test(): int {
    $arr = [];
    array_push($arr, 1);
    return $arr[0];
}
===expect===
