===description===
basic
===file===
<?php
function test(): void {
    $x = null;
    echo $x[0];
}
===expect===
NullArrayAccess@4:10: Cannot access array on null
