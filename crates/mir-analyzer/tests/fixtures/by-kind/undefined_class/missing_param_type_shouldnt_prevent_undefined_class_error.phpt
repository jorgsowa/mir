===description===
Missing param type shouldnt prevent undefined class error
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
/** @suppress MissingParamType */
function foo($s = Foo::BAR) : void {}
===expect===
UnusedPsalmSuppress@3:0-3:0: Suppress annotation for 'MissingParamType' is never used
UndefinedClass@3:19-3:22: Class Foo does not exist
