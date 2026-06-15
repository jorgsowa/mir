===description===
closure param type hint via use
===config===
suppress=UnusedVariable
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(Foo $x): void {};
===expect===
UndefinedClass@3:15-3:18: Class Vendor\Missing\Foo does not exist
