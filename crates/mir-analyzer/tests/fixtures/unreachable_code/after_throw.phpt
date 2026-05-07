===description===
after throw
===file===
<?php
function foo(): void {
    throw new RuntimeException('error');
    $x = 2;
}
===expect===
MissingThrowsDocblock@3:4: Exception RuntimeException is thrown but not declared in @throws
UnreachableCode@4:4: Unreachable code detected
