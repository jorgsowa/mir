===description===
InvalidArrayAssignment does NOT fire when assigning to an actual array.
===config===
suppress=UnusedVariable
===file===
<?php
$a = [];
$a[0] = 5;
===expect===
