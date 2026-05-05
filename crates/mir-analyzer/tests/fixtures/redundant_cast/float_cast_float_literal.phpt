===description===
Redundant cast from float literal to float

===file===
<?php
$x = (float)3.0;

===expect===
RedundantCast@2:12: Casting '3' to 'float' is redundant
