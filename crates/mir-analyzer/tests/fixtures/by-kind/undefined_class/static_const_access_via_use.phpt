===description===
static const access via use
===file===
<?php
use Vendor\Missing\Foo;
echo Foo::BAR;
===expect===
UndefinedClass@3:6-3:9: Class Vendor\Missing\Foo does not exist
