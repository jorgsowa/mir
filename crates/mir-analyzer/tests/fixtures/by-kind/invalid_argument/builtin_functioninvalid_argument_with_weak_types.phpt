===description===
Builtin functioninvalid argument with weak types
===config===
suppress=UnusedVariable
===file===
<?php
$s = substr(5, 4);
===expect===
InvalidArgument@2:13-2:14: Argument $string of substr() expects 'string', got '5'
