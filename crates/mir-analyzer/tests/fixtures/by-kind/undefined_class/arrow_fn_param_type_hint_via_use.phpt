===description===
arrow fn param type hint via use
===config===
suppress=UnusedVariable
===file===
<?php
use Vendor\Missing\Foo;
$fn = fn(Foo $x) => $x;
===expect===
UndefinedClass@3:9-3:12: Class Vendor\Missing\Foo does not exist
