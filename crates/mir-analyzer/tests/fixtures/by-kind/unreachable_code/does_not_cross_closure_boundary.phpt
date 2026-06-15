===description===
does not cross closure boundary
===file===
<?php
function foo(): void {
    return;
    $cb = function (): void {
        $x = 1;
    };
}
===expect===
UnreachableCode@4:4-6:6: Unreachable code detected
