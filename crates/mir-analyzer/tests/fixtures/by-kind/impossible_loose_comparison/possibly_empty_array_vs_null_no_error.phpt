===description===
A possibly-empty `array` is not disjoint from `null` — `[] == null` is true in
PHP (null converts to an empty array for the comparison), so this must not be
flagged, unlike a non-empty array (see nonempty_array_vs_null.phpt).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $arr): void {
    if ($arr == null) {}
}
===expect===
