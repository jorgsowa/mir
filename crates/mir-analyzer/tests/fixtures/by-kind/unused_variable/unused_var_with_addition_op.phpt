===description===
Unused var with addition op
===file===
<?php
$a = 5;
$a += 1;
===expect===
UnusedVariable@3:0-3:2: Variable $a is never read
