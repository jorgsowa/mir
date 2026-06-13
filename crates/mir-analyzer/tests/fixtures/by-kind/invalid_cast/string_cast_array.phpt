===description===
Invalid explicit cast from array to string

===config===
suppress=UnusedVariable
===file===
<?php
$x = (string)[];

===expect===
InvalidCast@2:14-2:16: Cannot cast 'array{}' to 'string'
