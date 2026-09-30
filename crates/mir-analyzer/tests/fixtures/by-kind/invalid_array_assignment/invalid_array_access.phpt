===description===
Invalid array access
===config===
suppress=UnusedVariable
===file===
<?php
$a = 5;
$a[0] = 5;
//<^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type '5'
===expect===
