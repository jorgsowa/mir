===description===
Define in both branches of conditional
===file===
<?php
$i = null;
//<^^ UnusedVariable: Variable $i is never read

if (($i = rand(0, 5)) || ($i = rand(0, 3))) {
    echo $i;
}
===expect===
