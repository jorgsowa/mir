===description===
Var reassigned in nested branches of if
===file===
<?php
$a = "foo";

if (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = "bar";
    } else {
        $a = "bat";
    }
} else {
    $a = "bang";
}

echo $a;
===expect===
UnusedVariable@2:1-2:3: Variable $a is never read
