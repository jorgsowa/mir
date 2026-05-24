===description===
unknown param type hint
===file===
<?php
function f(UnknownClass $x): void {}
===expect===
UndefinedClass@2:12: Class UnknownClass does not exist
UnusedParam@2:12: Parameter $x is never used
