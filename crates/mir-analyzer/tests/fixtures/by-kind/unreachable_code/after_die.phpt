===description===
after die
===file===
<?php
function foo(): void {
    die('fatal');
    $x = 2;
}
===expect===
UnreachableCode@4:4-4:11: Unreachable code detected
