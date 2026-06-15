===description===
Invalid explicit cast from array to string

===config===
suppress=UnusedVariable
===file===
<?php
$x = (string)[];

===expect===
InvalidCast@2:13-2:15: Cannot cast 'array{}' to 'string'
