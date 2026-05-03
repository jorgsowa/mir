===description===
after throw
===file===
<?php
function foo(): void {
    throw new RuntimeException('error');
    $x = 2;
}
===expect===
UnreachableCode@4:4: Unreachable code detected
===ignore===
TODO
