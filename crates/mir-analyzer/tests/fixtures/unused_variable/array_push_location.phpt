===description===
Verify UnusedVariable location for variable first assigned via array push.
===file===
<?php
function test(): void {
    $arr[] = 1;
}
===expect===
UnusedVariable@3:5: Variable $arr is never read
