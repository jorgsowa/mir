===file===
<?php
use Vendor\Missing\Foo;
$fn = fn(Foo $x) => $x;
===expect===
UndefinedClass: Class Vendor\Missing\Foo does not exist
