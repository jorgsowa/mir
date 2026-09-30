===description===
If var reassigned in branch with no use
===file===
<?php
$a = true;
//<^^ UnusedVariable: Variable $a is never read

if (rand(0, 1)) {
    $a = false;
}
===expect===
