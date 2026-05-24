===description===
closure return type hint via use
===file===
<?php
use Vendor\Missing\Foo;
$fn = function(): Foo {};
===expect===
UndefinedClass@3:19: Class Vendor\Missing\Foo does not exist
