===description===
Missing param type shouldnt prevent undefined class error
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
/** @suppress MissingParamType */
function foo($s = Foo::BAR) : void {}
===expect===
UndefinedClass@3:19: Class Foo does not exist
