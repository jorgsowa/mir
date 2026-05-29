===description===
after return
===file===
<?php
function foo(): int {
    return 1;
    $x = 2;
}
===expect===
UnreachableCode@4:5-4:12: Unreachable code detected
