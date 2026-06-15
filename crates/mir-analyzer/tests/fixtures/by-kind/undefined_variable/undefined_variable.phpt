===description===
Undefined variable
===config===
suppress=MissingClosureReturnType,UnusedVariable
===file===
<?php
$a = function() use ($i) {};
===expect===
UndefinedVariable@2:21-2:23: Variable $i is not defined
