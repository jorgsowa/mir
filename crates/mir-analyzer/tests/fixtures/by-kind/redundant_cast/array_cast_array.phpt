===description===
Redundant cast from array to array

===config===
suppress=UnusedVariable
===file===
<?php
$x = [];
$y = (array)$x;
//          ^^ RedundantCast: Casting 'array{}' to 'array' is redundant

===expect===
