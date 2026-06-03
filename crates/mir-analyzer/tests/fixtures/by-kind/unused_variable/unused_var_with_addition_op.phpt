===description===
Unused var with addition op
===file===
<?php
$a = 5;
$a += 1;
===expect===
UnusedVariable@3:1-3:3: Variable $a is never read
