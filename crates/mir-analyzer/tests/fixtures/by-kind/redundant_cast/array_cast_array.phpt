===description===
Redundant cast from array to array

===file===
<?php
$x = [];
$y = (array)$x;

===expect===
RedundantCast@3:13-3:15: Casting 'array{}' to 'array' is redundant
