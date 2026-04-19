===source===
<?php
function test(): void {
    $x = null;
    echo $x[0];
}
===expect===
NullArrayAccess: Cannot access array on null
