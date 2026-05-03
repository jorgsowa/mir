===description===
static method call via use
===file===
<?php
use Vendor\Missing\Foo;
Foo::bar();
===expect===
UndefinedClass: Class Vendor\Missing\Foo does not exist
===ignore===
TODO
