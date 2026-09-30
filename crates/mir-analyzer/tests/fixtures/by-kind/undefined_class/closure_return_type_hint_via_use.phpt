===description===
closure return type hint via use
===config===
suppress=UnusedVariable
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(): Foo {};
//                ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
===expect===
