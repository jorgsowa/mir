===description===
static property access via use
===file===
<?php
use Vendor\Missing\Foo;
echo Foo::$bar;
===expect===
UndefinedClass: Class Vendor\Missing\Foo does not exist
===ignore===
TODO
