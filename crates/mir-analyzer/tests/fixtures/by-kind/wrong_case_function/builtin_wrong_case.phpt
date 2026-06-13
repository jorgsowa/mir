===description===
Calling a built-in function with wrong casing is reported.
===config===
suppress=UnusedVariable
===file===
<?php
$x = STRLEN("hello");
===expect===
WrongCaseFunction@2:6-2:12: Function name 'STRLEN' has incorrect casing; use 'strlen'
