===description===
InvalidArrayAssignment fires for literal true.
===config===
suppress=UnusedVariable
===file===
<?php
$a = true;
$a[0] = 5;
===expect===
InvalidArrayAssignment@3:0-3:9: Cannot use [] assignment on non-array type 'true'
