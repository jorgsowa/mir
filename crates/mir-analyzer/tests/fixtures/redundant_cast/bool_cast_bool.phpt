===description===
Redundant cast from bool to bool

===file===
<?php
$x = true;
$y = (bool)$x;

===expect===
RedundantCast@3:11: Casting 'true' to 'bool' is redundant
