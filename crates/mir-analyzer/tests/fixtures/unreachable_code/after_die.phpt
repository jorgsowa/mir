===file===
<?php
function foo(): void {
    die('fatal');
    $x = 2;
}
===expect===
UnreachableCode: Unreachable code detected
