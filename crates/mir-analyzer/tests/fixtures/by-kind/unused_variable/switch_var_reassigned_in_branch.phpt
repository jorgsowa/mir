===description===
Switch var reassigned in branch
===file===
<?php
$a = false;
//<^^ UnusedVariable: Variable $a is never read

switch (rand(0, 2)) {
    case 0:
        $a = true;
}
===expect===
