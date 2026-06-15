===description===
Simple unused variable
===file===
<?php
$a = 5;
$b = [];
echo $a;
===expect===
UnusedVariable@3:0-3:2: Variable $b is never read
