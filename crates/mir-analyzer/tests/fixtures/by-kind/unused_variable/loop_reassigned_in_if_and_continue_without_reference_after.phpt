===description===
Loop reassigned in if and continue without reference after
===file===
<?php
$a = 5;
//<^^ UnusedVariable: Variable $a is never read

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = 7;
        continue;
    }

    $a = 3;
}
