===file===
<?php
function foo(): void {
    throw new RuntimeException('error');
    $x = 2;
}
===expect===
UnreachableCode: Unreachable code detected
