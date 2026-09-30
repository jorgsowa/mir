===description===
InvalidArrayAssignment fires for literal true.
===config===
suppress=UnusedVariable
===file===
<?php
$a = true;
$a[0] = 5;
//<^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type 'true'
===expect===
