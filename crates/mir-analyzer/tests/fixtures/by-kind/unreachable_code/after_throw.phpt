===description===
after throw
===file===
<?php
function foo(): void {
    throw new RuntimeException('error');
    $x = 2;
//  ^^^^^^^ UnreachableCode: Unreachable code detected
}
===expect===
