===description===
Redundant cast from array to array

===file===
<?php
$x = [];
$y = (array)$x;

===expect===
RedundantCast@3:13: Casting 'array{}' to 'array' is redundant
