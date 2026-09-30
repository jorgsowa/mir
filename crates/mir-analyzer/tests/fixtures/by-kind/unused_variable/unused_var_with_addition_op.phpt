===description===
Unused var with addition op
===file===
<?php
$a = 5;
$a += 1;
//<^^ UnusedVariable: Variable $a is never read
===expect===
