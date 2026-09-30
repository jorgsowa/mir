===description===
does not cross closure boundary
===file===
<?php
function foo(): void {
    return;
    $cb = function (): void {
//  ^ +2:6 UnreachableCode: Unreachable code detected
        $x = 1;
    };
}
===expect===
