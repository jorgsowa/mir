===description===
Verify UnusedVariable location for variable first assigned via array push.
===file===
<?php
function test(): void {
    $arr[] = 1;
//  ^^^^ UnusedVariable: Variable $arr is never read
}
===expect===
