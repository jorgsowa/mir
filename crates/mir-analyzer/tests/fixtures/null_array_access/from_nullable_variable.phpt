===source===
<?php
function test(bool $flag): void {
    $x = $flag ? [1, 2, 3] : null;
    echo $x[0];
}
===expect===
PossiblyNullArrayAccess: Cannot access array on possibly null value
