===description===
Verify UnusedVariable location for static variable declaration.
===file===
<?php
function test(): void {
    static $count;
}
===expect===
UnusedVariable@3:12-3:18: Variable $count is never read
