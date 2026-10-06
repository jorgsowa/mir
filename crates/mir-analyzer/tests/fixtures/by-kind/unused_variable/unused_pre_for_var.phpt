===description===
Unused pre for var
===file===
<?php
$i = 0;
//<^^ UnusedVariable: Variable $i is never read

for ($i = 0; $i < 10; $i++) {
    echo $i;
}
