===description===
static method call via use
===file===
<?php
use Vendor\Missing\Foo;
Foo::bar();
===expect===
UndefinedClass@3:1-3:4: Class Vendor\Missing\Foo does not exist
