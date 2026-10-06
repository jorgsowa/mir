===description===
Var reassigned in both branches of if
===file===
<?php
$a = "foo";
//<^^ UnusedVariable: Variable $a is never read

if (rand(0, 1)) {
    $a = "bar";
} else {
    $a = "bat";
}

echo $a;
