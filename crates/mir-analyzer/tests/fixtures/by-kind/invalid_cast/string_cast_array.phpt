===description===
Invalid explicit cast from array to string

===config===
suppress=UnusedVariable
===file===
<?php
$x = (string)[];
//           ^^ InvalidCast: Cannot cast 'array{}' to 'string'

===expect===
