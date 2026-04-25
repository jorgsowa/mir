===file===
<?php
function foo(): int {
    return 1;
    $x = 2;
}
===expect===
UnreachableCode: Unreachable code detected
