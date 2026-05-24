===description===
Redundant cast from int variable to int

===file===
<?php
$x = 3;
$y = (int)$x;

===expect===
RedundantCast@3:11: Casting '3' to 'int' is redundant
