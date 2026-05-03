===description===
closure param type hint via use
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(Foo $x): void {};
===expect===
UndefinedClass@3:15: Class Vendor\Missing\Foo does not exist
===ignore===
TODO
