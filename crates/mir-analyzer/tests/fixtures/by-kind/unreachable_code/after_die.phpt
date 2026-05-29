===description===
after die
===file===
<?php
function foo(): void {
    die('fatal');
    $x = 2;
}
===expect===
UnreachableCode@4:5-4:12: Unreachable code detected
