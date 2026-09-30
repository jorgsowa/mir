===description===
static const access via use
===file===
<?php
use Vendor\Missing\Foo;
echo Foo::BAR;
//   ^^^ UndefinedClass: Class Vendor\Missing\Foo does not exist
===expect===
