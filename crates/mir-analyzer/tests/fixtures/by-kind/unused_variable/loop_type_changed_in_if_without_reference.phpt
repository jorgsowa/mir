===description===
Loop type changed in if without reference
===file===
<?php
$a = false;
//<^^ UnusedVariable: Variable $a is never read

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = true;
    }
}
===expect===
