===description===
Var reassigned in both branches of if
===file===
<?php
$a = "foo";

if (rand(0, 1)) {
    $a = "bar";
} else {
    $a = "bat";
}

echo $a;
===expect===
UnusedVariable
===ignore===
TODO
