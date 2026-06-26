===description===
InvalidArrayAssignment does NOT fire for null — PHP allows subscript writes on null (auto-vivification).
===config===
suppress=UnusedVariable
===file===
<?php
$a = null;
$a[0] = 5;
===expect===
