===description===
after die
===file===
<?php
function foo(): void {
    die('fatal');
    $x = 2;
//  ^^^^^^^ UnreachableCode: Unreachable code detected
}
===expect===
