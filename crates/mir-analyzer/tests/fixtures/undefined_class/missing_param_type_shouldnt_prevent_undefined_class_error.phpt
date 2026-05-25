===description===
Missing param type shouldnt prevent undefined class error
===file===
<?php
/** @suppress MissingParamType */
function foo($s = Foo::BAR) : void {}
===expect===
UnusedParam@3:14: Parameter $s is never used
UndefinedClass@3:19: Class Foo does not exist
