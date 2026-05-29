===description===
after throw
===file===
<?php
function foo(): void {
    throw new RuntimeException('error');
    $x = 2;
}
===expect===
UnreachableCode@4:5-4:12: Unreachable code detected
