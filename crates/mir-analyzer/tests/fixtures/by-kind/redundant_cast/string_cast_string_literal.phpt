===description===
Redundant cast from string literal to string

===config===
suppress=UnusedVariable
===file===
<?php
$x = (string)"hello";

===expect===
RedundantCast@2:14-2:21: Casting '"hello"' to 'string' is redundant
