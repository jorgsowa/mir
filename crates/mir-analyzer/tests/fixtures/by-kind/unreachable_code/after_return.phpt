===description===
after return
===file===
<?php
function foo(): int {
    return 1;
    $x = 2;
//  ^^^^^^^ UnreachableCode: Unreachable code detected
}
===expect===
