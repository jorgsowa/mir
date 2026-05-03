===description===
static method call via use
===file===
<?php
use Vendor\Missing\Foo;
Foo::bar();
===expect===
UndefinedClass@3:0: Class Vendor\Missing\Foo does not exist
===ignore===
TODO
