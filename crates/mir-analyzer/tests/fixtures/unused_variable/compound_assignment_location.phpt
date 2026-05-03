===description===
Verify location tracking for compound assignment operators.
===file===
<?php
function test() {
    $x = 1;
    $x += 2;

    $y = "hello";
    $y .= "world";

    $z = 10;
    ++$z;

    $a = 1;
    $a++;
}
===expect===
UnusedVariable@6:4: Variable $y is never read
