===description===
A variable assigned inside both ternary branches is defined afterward —
whichever branch actually runs still performs a real assignment
===config===
suppress=UnusedVariable
===file===
<?php
function f(bool $c): void {
    $y = $c ? ($z = "yes") : ($z = "no");
    /** @mir-check $z is string */
    echo $z;
}
===expect===
