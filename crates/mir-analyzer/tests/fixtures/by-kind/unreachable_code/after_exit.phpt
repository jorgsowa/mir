===description===
after exit
===file===
<?php
function foo(): void {
    exit(1);
    $x = 2;
}
===expect===
UnreachableCode@4:5: Unreachable code detected
