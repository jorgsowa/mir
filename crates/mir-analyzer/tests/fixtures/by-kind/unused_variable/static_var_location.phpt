===description===
Verify UnusedVariable location for static variable declaration.
===file===
<?php
function test(): void {
    static $count;
}
===expect===
UnusedVariable@3:11-3:17: Variable $count is never read
