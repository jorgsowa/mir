===file===
<?php
function f(UnknownClass $x): void {}
===expect===
UnusedParam: Parameter $x is never used
UndefinedClass: Class UnknownClass does not exist
