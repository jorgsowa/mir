===description===
Redundant cast from array to array

===config===
suppress=UnusedVariable
===file===
<?php
$x = [];
$y = (array)$x;

===expect===
RedundantCast@3:12-3:14: Casting 'array{}' to 'array' is redundant
