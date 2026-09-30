===description===
Redundant cast from string literal to string

===config===
suppress=UnusedVariable
===file===
<?php
$x = (string)"hello";
//           ^^^^^^^ RedundantCast: Casting '"hello"' to 'string' is redundant

===expect===
