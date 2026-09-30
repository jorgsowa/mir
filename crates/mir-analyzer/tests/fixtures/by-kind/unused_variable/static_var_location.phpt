===description===
Verify UnusedVariable location for static variable declaration.
===file===
<?php
function test(): void {
    static $count;
//         ^^^^^^ UnusedVariable: Variable $count is never read
}
===expect===
