===description===
after exit
===file===
<?php
function foo(): void {
    exit(1);
    $x = 2;
//  ^^^^^^^ UnreachableCode: Unreachable code detected
}
===expect===
