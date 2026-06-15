===description===
closure return type hint via use
===config===
suppress=UnusedVariable
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(): Foo {};
===expect===
UndefinedClass@3:18-3:21: Class Vendor\Missing\Foo does not exist
