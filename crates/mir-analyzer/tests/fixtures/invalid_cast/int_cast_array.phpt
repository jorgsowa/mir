===description===
Invalid explicit cast from array to int

===file===
<?php
$x = (int)[];

===expect===
InvalidCast@2:10: Cannot cast 'array{}' to 'int'
