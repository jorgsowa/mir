===description===
Loop type changed in if and continue without reference
===file===
<?php
$a = false;
//<^^ UnusedVariable: Variable $a is never read

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = true;
        continue;
    }

    $a = false;
}
===expect===
