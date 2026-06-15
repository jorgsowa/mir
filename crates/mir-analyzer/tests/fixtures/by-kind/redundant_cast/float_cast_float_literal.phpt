===description===
Redundant cast from float literal to float

===config===
suppress=UnusedVariable
===file===
<?php
$x = (float)3.0;

===expect===
RedundantCast@2:12-2:15: Casting '3' to 'float' is redundant
