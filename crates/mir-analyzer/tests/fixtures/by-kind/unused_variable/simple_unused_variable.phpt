===description===
Simple unused variable
===file===
<?php
$a = 5;
$b = [];
echo $a;
===expect===
UnusedVariable@3:1-3:3: Variable $b is never read
