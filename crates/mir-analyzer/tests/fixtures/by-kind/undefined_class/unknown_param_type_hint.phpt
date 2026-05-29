===description===
unknown param type hint
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
function f(UnknownClass $x): void {}
===expect===
UndefinedClass@2:12-2:24: Class UnknownClass does not exist
