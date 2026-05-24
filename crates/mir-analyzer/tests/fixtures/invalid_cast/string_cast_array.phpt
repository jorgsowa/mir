===description===
Invalid explicit cast from array to string

===file===
<?php
$x = (string)[];

===expect===
InvalidCast@2:14: Cannot cast 'array{}' to 'string'
