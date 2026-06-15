===description===
Unused var with increment
===file===
<?php
$a = 5;
$a++;
===expect===
UnusedVariable@3:0-3:2: Variable $a is never read
