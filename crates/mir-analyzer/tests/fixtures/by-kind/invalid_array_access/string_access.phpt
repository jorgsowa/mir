===description===
InvalidArrayAccess fires when attempting array access on a string literal.
===config===
suppress=UnusedVariable
===file===
<?php
$s = "hello";
$c = $s[0];

===expect===
