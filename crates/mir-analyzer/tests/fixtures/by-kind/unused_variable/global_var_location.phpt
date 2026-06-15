===description===
Verify UnusedVariable location for global variable declaration.
===file===
<?php
function test(): void {
    global $config;
}
===expect===
UnusedVariable@3:11-3:18: Variable $config is never read
