===description===
unknown param type hint
===file===
<?php
function f(UnknownClass $x): void {}
===expect===
UndefinedClass@2:11: Class UnknownClass does not exist
UnusedParam@2:11: Parameter $x is never used
===ignore===
TODO
