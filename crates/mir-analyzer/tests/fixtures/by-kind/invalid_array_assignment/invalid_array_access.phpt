===description===
Invalid array access
===config===
suppress=UnusedVariable
===file===
<?php
$a = 5;
$a[0] = 5;
===expect===
InvalidArrayAssignment@3:1-3:10: Cannot use [] assignment on non-array type '5'
