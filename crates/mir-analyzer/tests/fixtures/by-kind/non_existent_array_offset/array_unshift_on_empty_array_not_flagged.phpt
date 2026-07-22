===description===
Same regression as array_push, for `array_unshift`: prepending onto a
provably empty array is identical to appending onto one (both just place
the pushed values in order, 0-indexed), so it must not leave the variable
stale at `array{}`.
===config===
suppress=UnusedParam
===file===
<?php
function test(): int {
    $arr = [];
    array_unshift($arr, 1);
    return $arr[0];
}
===expect===
