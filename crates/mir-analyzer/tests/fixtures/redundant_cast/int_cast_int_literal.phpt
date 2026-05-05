===description===
Redundant cast from int literal to int

===file===
<?php
$x = (int)3;

===expect===
RedundantCast@2:10: Casting '3' to 'int' is redundant
