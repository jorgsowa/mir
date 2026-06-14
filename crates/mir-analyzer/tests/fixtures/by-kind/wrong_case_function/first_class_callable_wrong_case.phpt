===description===
Wrong case in first-class callable syntax is reported.
===config===
suppress=UnusedVariable
===file===
<?php
function myFunc(int $x): int { return $x; }

$fn = MYFUNC(...);
===expect===
WrongCaseFunction@4:7-4:13: Function name 'MYFUNC' has incorrect casing; use 'myFunc'

