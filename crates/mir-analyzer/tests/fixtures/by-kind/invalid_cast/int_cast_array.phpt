===description===
Invalid explicit cast from array to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)[];
//        ^^ InvalidCast: Cannot cast 'array{}' to 'int'

===expect===
