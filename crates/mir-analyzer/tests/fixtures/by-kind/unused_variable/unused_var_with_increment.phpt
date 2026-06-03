===description===
Unused var with increment
===file===
<?php
$a = 5;
$a++;
===expect===
UnusedVariable@3:1-3:3: Variable $a is never read
