===description===
Calling a built-in function with wrong casing is reported.
===config===
suppress=UnusedVariable
===file===
<?php
$x = STRLEN("hello");
//   ^^^^^^ WrongCaseFunction: Function name 'STRLEN' has incorrect casing; use 'strlen'
===expect===
