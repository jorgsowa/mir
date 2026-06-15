===description===
unknown param type hint
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
function f(UnknownClass $x): void {}
===expect===
UndefinedClass@2:11-2:23: Class UnknownClass does not exist
