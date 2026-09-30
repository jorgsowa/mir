===description===
arrow fn param type hint via use
===config===
suppress=UnusedVariable
===file===
<?php
use Vendor\Missing\Foo;
$fn = fn(Foo $x) => $x;
//       ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
===expect===
