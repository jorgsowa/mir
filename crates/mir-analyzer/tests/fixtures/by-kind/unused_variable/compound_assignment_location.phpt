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
UnusedVariable@4:5-4:7: Variable $x is never read
UnusedVariable@6:5-6:7: Variable $y is never read
UnusedVariable@10:7-10:9: Variable $z is never read
UnusedVariable@13:5-13:7: Variable $a is never read
