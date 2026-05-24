===description===
Explicit int cast on int variable - should not emit ImplicitFloatToIntCast

===file===
<?php
$x = 3;
$y = (int)$x;

===expect===
RedundantCast@3:11: Casting '3' to 'int' is redundant
