===description===
static property access via use
===file===
<?php
use Vendor\Missing\Foo;
echo Foo::$bar;
//   ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
===expect===
