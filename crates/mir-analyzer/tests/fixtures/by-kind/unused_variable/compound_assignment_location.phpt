===description===
Verify location tracking for compound assignment operators.
===config===
suppress=MissingReturnType
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
UnusedVariable@4:4-4:6: Variable $x is never read
UnusedVariable@6:4-6:6: Variable $y is never read
UnusedVariable@10:6-10:8: Variable $z is never read
UnusedVariable@13:4-13:6: Variable $a is never read
