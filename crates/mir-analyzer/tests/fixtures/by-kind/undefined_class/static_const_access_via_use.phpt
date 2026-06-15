===description===
static const access via use
===file===
<?php
use Vendor\Missing\Foo;
echo Foo::BAR;
===expect===
UndefinedClass@3:5-3:8: Class Vendor\Missing\Foo does not exist
