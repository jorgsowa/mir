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
UnreachableCode@4:5-6:7: Unreachable code detected
