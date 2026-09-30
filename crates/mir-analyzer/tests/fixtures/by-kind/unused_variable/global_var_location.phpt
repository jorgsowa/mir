===description===
Verify UnusedVariable location for global variable declaration.
===file===
<?php
function test(): void {
    global $config;
//         ^^^^^^^ UnusedVariable: Variable $config is never read
}
===expect===
