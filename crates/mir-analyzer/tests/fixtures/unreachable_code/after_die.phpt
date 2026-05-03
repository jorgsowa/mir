===description===
after die
===file===
<?php
function foo(): void {
    die('fatal');
    $x = 2;
}
===expect===
UnreachableCode@4:4: Unreachable code detected
===ignore===
TODO
