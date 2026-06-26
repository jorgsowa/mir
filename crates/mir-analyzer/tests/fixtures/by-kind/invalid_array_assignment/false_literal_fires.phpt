===description===
InvalidArrayAssignment fires for literal false.
===config===
suppress=UnusedVariable
===file===
<?php
$a = false;
$a[0] = 5;
===expect===
InvalidArrayAssignment@3:0-3:9: Cannot use [] assignment on non-array type 'false'
