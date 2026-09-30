===description===
static method call via use
===file===
<?php
use Vendor\Missing\Foo;
Foo::bar();
//<^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
===expect===
