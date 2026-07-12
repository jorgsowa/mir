===description===
A static-method first-class-callable (`Foo::bar(...)`) on an undefined class
must report UndefinedClass, matching the plain `Foo::bar()` call form.
===config===
suppress=UnusedVariable
===file===
<?php
$c = MissingClass::baz(...);
===expect===
UndefinedClass@2:5-2:17: Class MissingClass does not exist
