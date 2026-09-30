===description===
Else var reassigned in branch and no reference
===file===
<?php
$a = true;
//<^^ UnusedVariable: Variable $a is never read

if (rand(0, 1)) {
    // do nothing
} else {
    $a = false;
}
===expect===
