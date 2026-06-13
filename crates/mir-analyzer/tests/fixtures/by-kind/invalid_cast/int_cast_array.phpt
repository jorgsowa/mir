===description===
Invalid explicit cast from array to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)[];

===expect===
InvalidCast@2:11-2:13: Cannot cast 'array{}' to 'int'
