===description===
Simple unused variable
===file===
<?php
$a = 5;
$b = [];
//<^^ UnusedVariable: Variable $b is never read
echo $a;
===expect===
