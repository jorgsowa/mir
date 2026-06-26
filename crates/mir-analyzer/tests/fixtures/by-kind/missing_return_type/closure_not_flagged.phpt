===description===
MissingReturnType (the function/interface-method check) does NOT fire for anonymous
functions or arrow functions. MissingClosureReturnType is a separate issue kind.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
$fn = function() { return 1; };
$arrow = fn() => 2;
===expect===
