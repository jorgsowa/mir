===description===
Redundant cast from string literal to string

===config===
suppress=UnusedVariable
===file===
<?php
$x = (string)"hello";

===expect===
RedundantCast@2:13-2:20: Casting '"hello"' to 'string' is redundant
