===description===
Switch var reassigned in branch with default
===file===
<?php
$a = false;
//<^^ UnusedVariable: Variable $a is never read

switch (rand(0, 2)) {
    case 0:
        $a = true;
        break;

    default:
        $a = false;
}
===expect===
