===description===
after exit
===file===
<?php
function foo(): void {
    exit(1);
    $x = 2;
}
===expect===
UnreachableCode@4:4-4:11: Unreachable code detected
