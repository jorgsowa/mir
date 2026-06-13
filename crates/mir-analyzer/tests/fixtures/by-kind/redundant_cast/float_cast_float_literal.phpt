===description===
Redundant cast from float literal to float

===config===
suppress=UnusedVariable
===file===
<?php
$x = (float)3.0;

===expect===
RedundantCast@2:13-2:16: Casting '3' to 'float' is redundant
