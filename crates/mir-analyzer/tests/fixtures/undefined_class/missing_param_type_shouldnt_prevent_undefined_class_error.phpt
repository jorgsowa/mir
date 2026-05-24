===description===
missingParamTypeShouldntPreventUndefinedClassError
===file===
<?php
/** @psalm-suppress MissingParamType */
function foo($s = Foo::BAR) : void {}
===expect===
UnusedParam@3:13: Parameter $s is never used
UndefinedClass@3:18: Class Foo does not exist
