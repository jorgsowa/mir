===file===
<?php
function foo(): void {
    exit(1);
    $x = 2;
}
===expect===
UnreachableCode: Unreachable code detected
