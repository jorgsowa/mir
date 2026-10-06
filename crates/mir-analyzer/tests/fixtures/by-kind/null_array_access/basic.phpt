===description===
Basic
===file===
<?php
function test(): void {
    $x = null;
    echo $x[0];
//       ^^^^^ NullArrayAccess: Cannot access array on null
}
