===description===
Verify UnusedVariable location for global variable declaration.
===file===
<?php
function test(): void {
    global $config;
}
===expect===
UnusedVariable@3:12-3:19: Variable $config is never read
