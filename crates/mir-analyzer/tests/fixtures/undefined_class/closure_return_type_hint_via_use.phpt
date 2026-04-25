===file===
<?php
use Vendor\Missing\Foo;
$fn = function(): Foo {};
===expect===
UndefinedClass: Class Vendor\Missing\Foo does not exist
