===description===
closure param type hint via use
===config===
suppress=UnusedVariable
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(Foo $x): void {};
//             ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
===expect===
