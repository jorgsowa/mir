===description===
Invalid explicit cast from array to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)[];

===expect===
InvalidCast@2:10-2:12: Cannot cast 'array{}' to 'int'
