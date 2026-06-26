===description===
InvalidArrayAssignment does NOT fire for string — PHP allows single-character string subscript writes.
===config===
suppress=UnusedVariable
===file===
<?php
$a = "hello";
$a[0] = 'x';
===expect===
