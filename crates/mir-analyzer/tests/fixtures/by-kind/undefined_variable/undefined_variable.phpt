===description===
Undefined variable
===config===
suppress=MissingClosureReturnType,UnusedVariable
===file===
<?php
$a = function() use ($i) {};
//                   ^^ UndefinedVariable: Variable $i is not defined
===expect===
